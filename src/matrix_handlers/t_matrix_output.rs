use std::fmt::Display;

/// Defines the output of a Matrix-capable command.

pub struct MatrixOutput {
    pub matrix_header: Option<MatrixHeader>,
    pub matrix_footer: Option<MatrixFooter>,
    pub matrices: Vec<Matrix>
}

/// Defines the overall header for all the matrices - the report header
pub struct MatrixHeader {
    /// May contain line breaks; some output formats may render these as an array.
    pub title: Option<String>,

    /// If true, causes the program header (JAWS + revision information)
    pub output_program_header: bool
}

/// Defines the footer for all the matrices.
pub struct MatrixFooter {
    /// May contain line breaks; some output formats may render these as an array.
    pub footer: Option<String>,
    /// If true, causes the program footer (run by, run date etc.) to be output
    pub output_program_footer: bool
}

/// A coherent set of data for output.
pub struct Matrix {
    /// May contain line breaks; some output formats may render these as an array.
    pub header: Vec<String>,

    /// Row data can be any type that implements Display, hence it needs to be Boxed, since
    /// Display is a trait and the size of the underlying object cannot be known at compile-time.
    /// Jagged rows are not allowed.  If a cell is not filled, the Option should be None.
    pub rows: Vec<Vec<Option<Box<dyn Display>>>>,

    /// Aggregate rows are data that appear at the end of a table which are computed by some
    /// calculation of that table, e.g. column totals.  They are generated separately.
    /// Output routines will attempt to display these in the way most appropriate for their
    /// type.
    pub aggregate_rows: Option<Vec<MatrixAggregateValue>>,

    /// Any notes that should appear with this dataset.
    pub notes: Option<Vec<String>>,

    /// If true, the first rows row is a header
    pub first_rows_header: bool,
}

/// An aggregation of values found in a Matrix (e.g. Total, Average etc.).
pub struct MatrixAggregateValue {
    /// The name of this aggregate
    pub name: String,
    /// The value of the aggregate.  This must have a value.
    pub value: Box<dyn Display>,
}
