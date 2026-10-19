//! The columns of the list of places or plans within a kind, named by the
//! core so the page shows what the ranking measured.

use serde::{Deserialize, Serialize};

use crate::Text;

use super::Arena;

/// A column of the list of places or plans within a kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum ColumnId {
    /// What is left at the end, after tax: what the list is ranked by.
    LeftAfterTax,
    /// What the place or plan loses a year, as a fund's fee is stated.
    CostAYear,
    /// Fees paid over the years (the index kinds' lists only).
    Fees,
    /// Tax paid on the gain.
    Tax,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Column {
    pub id: ColumnId,
    pub name: Text,
}

/// The columns of the list within an arena, in the order shown. The safe
/// kinds' places differ by what they leave, their yearly cost and the tax;
/// the index kinds' plans also show the fees.
#[must_use]
pub fn columns_for(arena: Arena) -> Vec<Column> {
    let column = |id, en, he| Column {
        id,
        name: Text::new(en, he),
    };
    let mut columns = vec![
        column(ColumnId::LeftAfterTax, "Left after tax", "נשאר אחרי מס"),
        column(ColumnId::CostAYear, "Cost a year", "עלות בשנה"),
    ];
    // A fund's or a bank's cost is the yearly one; fees are paid to brokers.
    if arena == Arena::Index {
        columns.push(column(ColumnId::Fees, "Fees", "עמלות"));
    }
    columns.push(column(ColumnId::Tax, "Tax", "מס"));
    columns
}
