use async_trait::async_trait;
use aws_sdk_ec2::types::{Instance, RecurringCharge, RecurringChargeFrequency, ReservedInstances};
use aws_smithy_types_convert::date_time::DateTimeExt;
use chrono::{DateTime, Utc};
use tabled::settings::Alignment;
use tabled::settings::object::Columns;
use tabled::Table;

use crate::{Options, SubCommands};
use crate::aws_handler::AWSHandler;
use crate::commands::{Command, notify_clear, notify_comms, notify_working};
use crate::commands::ec2::EC2Command;
use crate::errors::jaws_error::JawsError;
use crate::tabulatable::Tabulatable;
use crate::textutils::{center_text, report_title};

const SECONDS_PER_YEAR: i32 = 60 * 60 * 24 * 365;
const HOURS_PER_YEAR: i32 = 24 * 365;

pub struct ResCommand {}

impl ResCommand
{
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for ResCommand
{
    async fn run(&mut self, options: &mut Options) -> Result<(), JawsError> {
        let mut handler = AWSHandler::new(options).await;

        notify_comms(Some("Getting reservation data".to_string()));

        // Get all reservations

        let reservations_result = handler.reservations_get_live().await;

        // Check we got a a good result (and return early if not)

        let mut reservations = match reservations_result {
            Ok(reservations) => reservations,
            Err(err) => return Err(err)
        };

        // Otherwise we have a good list of reservations.

        if reservations.len() == 0 {
            notify_clear();
            println!("No active reservations found.");
            return Ok(());
        }

        // Create a model now which will support output in tabular form, the same as Jaws-1,
        // and also as a one-shot Prometheus output type, which can be consumed and sent to
        // Otel.

        notify_working();
        let model = calculate_model(&reservations, &mut handler).await;
        notify_clear();

        // .. and output it
        report_title(format!("EC2 Reservations ({})", model.elements.len()));
        dump_model_tabular(&model);

        // If --show-unused is present, get all EC2 instances and thin out the reservations.

        if let SubCommands::RES { show_unused } = options.subcommand {
            if show_unused {
                let instance_result = handler.ec2_get_all().await?;
                let uncovered_instances = thin_reservations(&instance_result, &mut reservations);

                // Calculate and dump the unused reservations  (or a "none" string if there aren't any).
                let unused_model = calculate_model(&reservations, &mut handler).await;
                println!();
                report_title(format!("Unused EC2 Reservations ({})", unused_model.elements.len()));
                if reservations.len() > 0 {
                    dump_model_tabular(&unused_model);
                } else {
                    println!("{}", center_text("** NONE **".to_string()));
                }

                // Output the uncovered instances, if there are any.as
                println!();
                report_title(format!("Uncovered EC2 Instances ({})", uncovered_instances.len()));
                if uncovered_instances.len() > 0 {
                    let mut ec2_command: EC2Command = EC2Command::new();
                    options.wide = true;
                    ec2_command.run_with_filter(uncovered_instances, options).await;
                } else {
                    println!("{}", center_text("** NONE **".to_string()));
                }

            }
        }

        Ok(())
    }
}

fn thin_reservations(instances: &Vec<Instance>, reservations: &mut Vec<ReservedInstances>) -> Vec<String> {
    // The idea of this function is to remove single reservations as each instance "consumes"
    // them, leaving the reservations vec containing only unused reservations.

    let mut uncovered: Vec<String> = Vec::new();

    'instances: for instance in instances.iter().by_ref() {
        // Find a reservation that applies to this instance.
        for reservation in reservations.iter_mut() {
            if instance.instance_type.as_ref().unwrap() == reservation.instance_type.as_ref().unwrap() {
                // This reservation applies to this instance.

                // If this reservation has any more instances in it, decrement them. Otherwise,
                // move on to the next instance.

                if reservation.instance_count.unwrap() > 0 {
                    reservation.instance_count = Some(reservation.instance_count.unwrap() - 1);
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

    uncovered
}

fn dump_model_tabular(model: &CalculationModel) {
    (model as &dyn Tabulatable).tabulate(false);
}

impl Tabulatable for CalculationModel {
    fn get_table_headers(&self, _extended: bool) -> Vec<String> {
        vec!["Type".to_string(), "#".to_string(), "AZ".to_string(), "Expiry".to_string(),
             "Days".to_string(), "Term Yrs".to_string(), "Model".to_string(), "$ Res / Hr".to_string(),
             "$ Res Fixed".to_string(), "$ Res Yearly".to_string(), "$ ODM / Hr".to_string(), "$ ODM Yearly".to_string(),
             "$ Saving Yearly".to_string()]
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
        let row = vec![empty.clone(), empty.clone(), empty.clone(), empty.clone(), empty.clone(), empty.clone(), empty.clone(),
                           empty.clone(),
                           "Total".to_string(),
                           format_money(self.total_actual_yearly),
                           empty.clone(),
                           format_money(self.total_odm_yearly),
                           format_money(self.total_odm_yearly - self.total_actual_yearly)];

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

async fn calculate_model(reservations: &Vec<ReservedInstances>, handler: &mut AWSHandler) -> CalculationModel {
    let mut elements = Vec::new();

    let mut total_odm_yearly: f32 = 0.0;
    let mut total_res_yearly: f32 = 0.0;

    for res in reservations {

        // We need to precompute some values because they are used in calculation further down
        // the struct.

        let res_yearly: f32 = sum_recurring_charges(res.recurring_charges()) * (HOURS_PER_YEAR as f32) * res.instance_count.unwrap() as f32;
        let odm_yearly: f32 = handler.get_odm_rate(res.instance_type().unwrap()).await * (HOURS_PER_YEAR as f32) * res.instance_count.unwrap() as f32;

        elements.push(ReservationElement {
            name: String::from(res.instance_type().unwrap().as_str()),
            qty: res.instance_count.unwrap(),
            az: String::from(res.availability_zone().unwrap_or("None".to_string().as_str())),
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

    return CalculationModel {
        elements,
        total_odm_yearly: total_odm_yearly,
        total_actual_yearly: total_res_yearly,
    };
}

fn sum_recurring_charges(charges: &[RecurringCharge]) -> f32 {
    let mut sum: f32 = 0.0;

    for charge in charges {
        // Currently there is only Hourly
        sum = sum + match charge.frequency().unwrap() {
            RecurringChargeFrequency::Hourly => charge.amount().unwrap() as f32,
            _ => panic!("Recurring charge frequency {} unknown!",
                        charge.frequency().unwrap().as_str())
        };
    }

    sum
}

fn days_remaining(dt: DateTime<Utc>) -> i64 {
    let diff = dt - Utc::now();
    diff.num_days()
}


fn format_money(amount: f32) -> String {
    return format!("{:.2}", amount);
}
