use std::fmt::Display;

/// TODO
pub struct MatrixOutput {
    matrix_header: Option<MatrixHeader>,
    matrix_fotter: Option<MatrixFooter>,
    matrices: Vec<Matrix>
}


/// Defines the overall header for all the matrices - the report header
pub struct MatrixHeader {
    /// May contain line breaks; some output formats may render these as an array.
    title: String,
}

/// Defines the footer for all the matrices.
pub struct MatrixFooter {
    /// May contain line breaks; some output formats may render these as an array.
    footer: String
}

/// A coherent set of data for output.
pub struct Matrix {
    /// May contain line breaks; some output formats may render these as an array.
    header: Vec<String>,

    /// Row data can be any type that implements Display, hence it needs to be Boxed, since
    /// Display is a trait and the size of the underlying object cannot be known at compile-time.
    /// Jagged rows are not allowed.  If a cell is not filled, the Option should be None.
    rows: Vec<Vec<Option<Box<dyn Display>>>>,

    /// Aggregate rows are data that appear at the end of a table which are computed by some
    /// calculation of that table, e.g. column totals.  They are generated separately.
    /// Output routines will attempt to display these in the way most appropriate for their 
    /// type.
    aggregate_rows: Vec<Vec<MatrixAggregateValue>>,

    /// Any notes that should appear with this dataset.
    notes: Vec<String>
}

/// An aggregation of values found in a Matrix (e.g. Total, Average etc.).
pub struct MatrixAggregateValue {
    /// The name of this aggregate 
    name: String,
    /// The value of the aggregate.  This must have a value.
    value: Box<dyn Display>,
}

