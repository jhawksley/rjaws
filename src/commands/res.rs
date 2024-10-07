use async_trait::async_trait;
use aws_sdk_ec2::types::{Instance, RecurringCharge, RecurringChargeFrequency, ReservedInstances};
use aws_smithy_types_convert::date_time::DateTimeExt;
use chrono::{DateTime, Utc};
use rust_decimal::prelude::*;
use rusty_money::{iso, Money, Round};
use sprintf::sprintf;
use std::fmt::Display;
use tabled::settings::object::Columns;
use tabled::settings::Alignment;
use tabled::Table;

use crate::commands::ec2::EC2Command;
use crate::errors::jaws_error::JawsError;
use crate::matrix_handlers::t_matrix_output::{
    Matrix, MatrixAggregateRowT, MatrixAggregateValue, MatrixFooter, MatrixHeader, MatrixOutput,
    MatrixRowT, MatrixRowsT,
};
use crate::t_aws_handler::AWSHandler;
use crate::t_command::Command;
use crate::t_tabulatable::Tabulatable;
use crate::textutils::Textutil;
use crate::{Options, SubCommands};

const SECONDS_PER_YEAR: i32 = 60 * 60 * 24 * 365;
const HOURS_PER_YEAR: i32 = 24 * 365;

pub struct ResCommand {
    model: Option<CalculationModel>,
    unused_model: Option<CalculationModel>,
    uncovered_instances_matrix: Option<MatrixOutput>,
    covered_instances_matrix: Option<MatrixOutput>,
    wide: bool,
}

unsafe impl Send for ResCommand {}

impl ResCommand {
    pub fn new() -> Self {
        Self {
            model: None,
            unused_model: None,
            uncovered_instances_matrix: None,
            covered_instances_matrix: None,
            wide: false,
        }
    }

    fn get_reservations_matrix(&self) -> Matrix {
        let mut main_rows: MatrixRowsT = Vec::new();
        let mut header: Vec<Option<Box<dyn Display>>> = Vec::new();
        header.push(Some(Box::new("Type".to_string())));
        header.push(Some(Box::new("#".to_string())));
        header.push(Some(Box::new("AZ".to_string())));
        header.push(Some(Box::new("Expiry".to_string())));
        header.push(Some(Box::new("Days".to_string())));
        header.push(Some(Box::new("Term Yrs".to_string())));
        header.push(Some(Box::new("Model".to_string())));
        header.push(Some(Box::new("$ Res/Hr".to_string())));
        header.push(Some(Box::new("$ Res/Fixed".to_string())));
        header.push(Some(Box::new("$ Res/Year".to_string())));
        header.push(Some(Box::new("$ ODM/Hr".to_string())));
        header.push(Some(Box::new("$ ODM/Year".to_string())));
        header.push(Some(Box::new("$ Saving/Year".to_string())));

        main_rows.push(header);

        let mut total_res_count: i32 = 0;
        let mut total_res_expenditure_year: f32 = 0.0;
        let mut total_res_saving: f32 = 0.0;

        for element in &self.model.as_ref().unwrap().elements {
            let mut row: MatrixRowT = Vec::new();
            row.push(Some(Box::new(element.name.clone())));
            row.push(Some(Box::new(element.qty.to_string())));
            row.push(Some(Box::new(element.az.clone())));
            row.push(Some(Box::new(element.expiry.to_string())));
            row.push(Some(Box::new(element.days_remaining.to_string())));
            row.push(Some(Box::new(element.term_years.to_string())));
            row.push(Some(Box::new(element.res_model.to_string())));
            row.push(Some(Box::new(format_money(element.res_recurring))));
            row.push(Some(Box::new(format_money(element.res_fixed))));
            row.push(Some(Box::new(format_money(element.res_yearly))));
            row.push(Some(Box::new(format_money(element.odm_rate))));
            row.push(Some(Box::new(format_money(element.odm_yearly))));
            row.push(Some(Box::new(format_money(element.saving_yearly))));
            main_rows.push(row);

            total_res_count += element.qty;
            total_res_expenditure_year += element.res_yearly;
            total_res_saving += element.saving_yearly;
        }

        let aggregate_rows = Some(vec![
            MatrixAggregateValue {
                name: "Total Reservations".to_string(),
                value: Box::new((total_res_count)),
            },
            MatrixAggregateValue {
                name: "Total Yearly Spend".to_string(),
                value: Box::new((format_money(total_res_expenditure_year))),
            },
            MatrixAggregateValue {
                name: "Total Yearly Saving".to_string(),
                value: Box::new((format_money(total_res_saving))),
            },
        ]);

        Matrix {
            header: Some(vec!["Active Reservations".to_string()]),
            rows: Some(main_rows),
            aggregate_rows: aggregate_rows, // Some(aggregate_rows),
            notes: None,
            first_rows_header: true,
        }
    }
}

#[async_trait]
impl Command for ResCommand {
    async fn run(&mut self, options: &mut Options) -> Result<(), JawsError> {
        let mut handler = AWSHandler::new(options).await;

        let textutil = Textutil::new(options);

        textutil.notify_comms(Some("Getting reservation data".to_string()));

        // Get all reservations

        let reservations_result = handler.reservations_get_live().await;

        // Check we got a a good result (and return early if not)

        let mut reservations = match reservations_result {
            Ok(reservations) => reservations,
            Err(err) => return Err(err),
        };

        // Otherwise we have a good list of reservations.

        if reservations.len() == 0 {
            textutil.notify_clear();
            println!("No active reservations found.");
            return Ok(());
        }

        // Create a model now which will support output in tabular form, the same as Jaws-1.

        textutil.notify_working();
        self.model = Some(calculate_model(&reservations, &mut handler).await);
        textutil.notify_clear();

        // .. and output it
        // textutil.report_title(format!("EC2 Reservations ({})", model.elements.len()));
        // dump_model_tabular(&model);

        // If --show-unused is present, get all EC2 instances and thin out the reservations.
        if let SubCommands::RES { show_unused: true } = options.subcommand {
            self.wide = true;
            let mut instance_result = handler.ec2_get_all().await?;

            // remove Terminated instances for this command.
            instance_result.retain(|x| x.state().unwrap().name().unwrap().as_str() == "running");
            let (covered_instances, uncovered_instances) =
                thin_reservations(&instance_result, &mut reservations);

            // Calculate and dump the unused reservations  (or a "none" string if there aren't any).
            self.unused_model = Some(calculate_model(&reservations, &mut handler).await);

            options.wide = true;

            let mut ec2_command = EC2Command::new(&options).await;

            ec2_command
                .run_with_filter(uncovered_instances, options)
                .await;
            self.uncovered_instances_matrix = ec2_command.get_matrix_output();

            ec2_command
                .run_with_filter(covered_instances, options)
                .await;
            self.covered_instances_matrix = ec2_command.get_matrix_output();
        }

        Ok(())
    }

    fn get_matrix_output(&mut self) -> Option<MatrixOutput> {
        // We need to take care here of the options.
        // There will always by the standard res command output - and this never changes (--wide is not
        // supported here).  But if the user selects --show-unused, we will show coverage data
        // and this is drawn from the EC2 command.  This _does_ honour --wide, and will produce
        // matrices of its own.  These must be integrated into `res`'s output.

        let reservations_matrix = self.get_reservations_matrix();

        let mut matrices: Vec<Matrix> = vec![reservations_matrix];

        if self.wide {
            while let Some(mut matrix) = self
                .covered_instances_matrix
                .as_mut()
                .unwrap()
                .matrices
                .pop()
            {
                matrix.header = Some(vec!["Covered Instances".to_string()]);
                matrices.push(matrix)
            }
            while let Some(mut matrix) = self
                .uncovered_instances_matrix
                .as_mut()
                .unwrap()
                .matrices
                .pop()
            {
                matrix.header = Some(vec!["Uncovered Instances".to_string()]);
                matrices.push(matrix)
            }
        }

        Some(MatrixOutput {
            matrix_header: Some(MatrixHeader {
                title: Some("EC2 Reservations".to_string()),
                output_program_header: true,
            }),
            matrix_footer: Some(MatrixFooter {
                footer: None,
                output_program_footer: true,
            }),
            matrices: matrices,
        })
    }
}

async fn output_table(
    instances: Vec<String>,
    title: &str,
    options: &mut Options,
    textutil: &Textutil,
    ec2_command: &mut EC2Command,
) {
    // Output the uncovered instances, if there are any.
    println!();
    textutil.report_title(sprintf!(title, instances.len()).unwrap());

    if instances.len() > 0 {
        options.wide = true;
        ec2_command.run_with_filter(instances, options).await;
    } else {
        println!("{}", textutil.center_text("** NONE **".to_string()));
    }
}

fn thin_reservations(
    instances: &Vec<Instance>,
    reservations: &mut Vec<ReservedInstances>,
) -> (Vec<String>, Vec<String>) {
    // The idea of this function is to remove single reservations as each instance "consumes"
    // them, leaving the reservations vec containing only unused reservations.

    let mut uncovered: Vec<String> = Vec::new();
    let mut covered: Vec<String> = Vec::new();

    'instances: for instance in instances.iter().by_ref() {
        // Find a reservation that applies to this instance.
        for reservation in reservations.iter_mut() {
            if instance.instance_type.as_ref().unwrap()
                == reservation.instance_type.as_ref().unwrap()
            {
                // This reservation applies to this instance.

                // If this reservation has any more instances in it, decrement them. Otherwise,
                // move on to the next instance.

                if reservation.instance_count.unwrap() > 0 {
                    reservation.instance_count = Some(reservation.instance_count.unwrap() - 1);
                    covered.push(instance.instance_id.as_ref().unwrap().clone());
                    continue 'instances;
                }
                // else continue the next reservation.
            }
        }

        // If we get this far, we exhausted all the reservations - this is an uncovered instance.
        uncovered.push(instance.instance_id.as_ref().unwrap().clone());
    }

    // Retain instances in the reservations vec with counts > 0
    reservations.retain(|x| x.instance_count.unwrap() > 0);

    (covered, uncovered)
}

fn dump_model_tabular(model: &CalculationModel) {
    (model as &dyn Tabulatable).tabulate(false);
}

impl Tabulatable for CalculationModel {
    fn get_table_headers(&self, _extended: bool) -> Vec<String> {
        vec![
            "Type".to_string(),
            "#".to_string(),
            "AZ".to_string(),
            "Expiry".to_string(),
            "Days".to_string(),
            "Term Yrs".to_string(),
            "Model".to_string(),
            "$ Res / Hr".to_string(),
            "$ Res Fixed".to_string(),
            "$ Res Yearly".to_string(),
            "$ ODM / Hr".to_string(),
            "$ ODM Yearly".to_string(),
            "$ Saving Yearly".to_string(),
        ]
    }

    fn get_table_rows(&self, _extended: bool) -> Vec<Vec<String>> {
        let mut rows: Vec<Vec<String>> = Vec::new();

        for res in self.elements.iter().by_ref() {
            let mut row: Vec<String> = Vec::new();
            row.push(res.name.clone());
            row.push(res.qty.to_string());
            row.push(res.az.clone());
            row.push(res.expiry.to_string());
            row.push(res.days_remaining.to_string());
            row.push(res.term_years.to_string());
            row.push(res.res_model.clone());
            row.push(format_money(res.res_recurring));
            row.push(format_money(res.res_fixed));
            row.push(format_money(res.res_yearly));
            row.push(format_money(res.odm_rate));
            row.push(format_money(res.odm_yearly));
            row.push(format_money(res.saving_yearly));

            rows.push(row);
        }

        // Totals
        let empty: String = "".to_string();
        let row = vec![
            empty.clone(),
            empty.clone(),
            empty.clone(),
            empty.clone(),
            empty.clone(),
            empty.clone(),
            empty.clone(),
            empty.clone(),
            "Total".to_string(),
            format_money(self.total_actual_yearly),
            empty.clone(),
            format_money(self.total_odm_yearly),
            format_money(self.total_odm_yearly - self.total_actual_yearly),
        ];

        rows.push(vec![]);
        rows.push(row);

        rows
    }

    fn modify(&self, table: &mut Table) {
        table.modify(Columns::single(1), Alignment::right());
        table.modify(Columns::new(7..), Alignment::right());
    }
}

struct CalculationModel {
    // Array of structs, one per reservation type
    // type, number, AZ (if tied), Expiry, Days Remaining, Term Years, Resv. Model, Recurring fee,
    // Resv. fixed fee, ODM Rate, Yearly ODM, Yearly Actual, Saving
    // Total Yearly ODM, total Actual, total saving.
    elements: Vec<ReservationElement>,
    total_odm_yearly: f32,
    total_actual_yearly: f32,
}

struct ReservationElement {
    name: String,
    qty: i32,
    az: String,
    expiry: DateTime<Utc>,
    days_remaining: i64,
    term_years: i64,
    res_model: String,
    res_recurring: f32,
    res_fixed: f32,
    res_yearly: f32,
    odm_rate: f32,
    odm_yearly: f32,
    saving_yearly: f32,
}

async fn calculate_model(
    reservations: &Vec<ReservedInstances>,
    handler: &mut AWSHandler,
) -> CalculationModel {
    let mut elements = Vec::new();

    let mut total_odm_yearly: f32 = 0.0;
    let mut total_res_yearly: f32 = 0.0;

    for res in reservations {
        // We need to precompute some values because they are used in calculation further down
        // the struct.

        let res_yearly: f32 = sum_recurring_charges(res.recurring_charges())
            * (HOURS_PER_YEAR as f32)
            * res.instance_count.unwrap() as f32;
        let odm_yearly: f32 = handler.get_odm_rate(res.instance_type().unwrap()).await
            * (HOURS_PER_YEAR as f32)
            * res.instance_count.unwrap() as f32;

        elements.push(ReservationElement {
            name: String::from(res.instance_type().unwrap().as_str()),
            qty: res.instance_count.unwrap(),
            az: String::from(
                res.availability_zone()
                    .unwrap_or("None".to_string().as_str()),
            ),
            expiry: res.end().unwrap().to_chrono_utc().unwrap(),
            days_remaining: days_remaining(res.end().unwrap().to_chrono_utc().unwrap()),
            term_years: res.duration().unwrap() / SECONDS_PER_YEAR as i64,
            res_model: res.offering_type().unwrap().to_string(),
            res_recurring: sum_recurring_charges(res.recurring_charges()),
            res_fixed: res.fixed_price.unwrap(),
            res_yearly: res_yearly,
            odm_rate: handler.get_odm_rate(res.instance_type().unwrap()).await,
            odm_yearly: odm_yearly,
            saving_yearly: odm_yearly - res_yearly,
        });

        total_odm_yearly = total_odm_yearly + odm_yearly;
        total_res_yearly = total_res_yearly + res_yearly;
    }

    CalculationModel {
        elements,
        total_odm_yearly: total_odm_yearly,
        total_actual_yearly: total_res_yearly,
    }
}

fn sum_recurring_charges(charges: &[RecurringCharge]) -> f32 {
    let mut sum: f32 = 0.0;

    for charge in charges {
        // Currently there is only Hourly
        sum = sum
            + match charge.frequency().unwrap() {
                RecurringChargeFrequency::Hourly => charge.amount().unwrap() as f32,
                _ => panic!(
                    "Recurring charge frequency {} unknown!",
                    charge.frequency().unwrap().as_str()
                ),
            };
    }

    sum
}

fn days_remaining(dt: DateTime<Utc>) -> i64 {
    let diff = dt - Utc::now();
    diff.num_days()
}

fn format_money(amount: f32) -> String {
    let usd = Money::from_decimal(Decimal::try_from(amount).unwrap(), iso::USD);
    usd.round(2, Round::HalfUp).to_string()
}
