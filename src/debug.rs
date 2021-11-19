/*
* Copyright 2021 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

//! Handles debugging information.
//!
//! There are a few basic position helpers: [`Line`], [`Lines`], and [`DbgPos`]. This last one
//! represents a position in a file.
//!
//! Then [`DbgNode`] essentially corresponds to a cell and handles two things:
//! - a map from the cell data offsets to [`DbgNode`]s, and
//! - a vector storing the [`DbgNode`]s corresponding to the cell's cell-references.
//!
//! Last, [`DbgInfo`] stores the [`DbgNode`]s for all cells thanks to a map which keys are the
//! (string) representation hashes of the cells of the input program.

use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};
use ton_types::{Cell, UInt256};

/// Alias for a [`Vec`] of [`Line`].
pub type Lines = Vec<Line>;

/// Content of a line ([`String`]) and [`DbgPos`] information.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// Content of the line.
    pub text: String,
    /// Position information.
    pub pos: DbgPos,
}
impl Line {
    /// Constructor.
    pub fn new(text: &str, filename: &str, line: usize) -> Self {
        Line {
            text: String::from(text),
            pos: DbgPos {
                filename: String::from(filename),
                line,
                line_code: line,
            },
        }
    }

    /// Constructor with a custom line code.
    pub fn new_extended(text: &str, filename: &str, line: usize, line_code: usize) -> Self {
        Line {
            text: String::from(text),
            pos: DbgPos {
                filename: String::from(filename),
                line,
                line_code,
            },
        }
    }
}

pub fn lines_to_string(lines: &Lines) -> String {
    lines
        .iter()
        .fold(String::new(), |result, line| result + line.text.as_str())
}

/// Position information for lines.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DbgPos {
    /// Name of the source file, empty for *none*.
    pub filename: String,
    /// Line number.
    pub line: usize,
    /// Line code, ignored in serialization and printing.
    #[serde(skip)]
    pub line_code: usize,
}
impl fmt::Display for DbgPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let filename = if self.filename.is_empty() {
            "<none>"
        } else {
            &self.filename
        };
        write!(f, "{}:{}", filename, self.line)
    }
}
impl Default for DbgPos {
    fn default() -> Self {
        Self {
            filename: String::new(),
            line: 0,
            line_code: 0,
        }
    }
}

/// A map from offsets (for cell data) to position information.
pub type OffsetPos = BTreeMap<usize, DbgPos>;

/// Information about the components of a node (cell).
#[derive(Clone)]
pub struct DbgNode {
    /// Maps data-offsets to position information.
    pub offsets: OffsetPos,
    /// List of children (cell references) position information.
    pub children: Vec<DbgNode>,
}
impl DbgNode {
    /// Constructs an empty node.
    pub fn new() -> Self {
        Self {
            offsets: OffsetPos::new(),
            children: vec![],
        }
    }

    /// Constructs a node with the data at offset `0` associated to `pos`.
    pub fn from(pos: DbgPos) -> Self {
        let mut node = Self::new();
        node.offsets.insert(0, pos);
        node
    }

    /// Registers an `offset`/`pos`ition association.
    ///
    /// **Overwrites** the previous binding, if any.
    ///
    /// # TODO
    ///
    /// - Check the binding is new, or at least return the old one so that caller can check/ignore
    ///   previous bindings?
    pub fn append(self: &mut Self, offset: usize, pos: DbgPos) {
        self.offsets.insert(offset, pos);
    }

    /// Merges `self` with the content (offsets and children) of `dbg` with an `offset`.
    ///
    /// If self contains conflicting `offsets` bindings, they will be overwritten.
    ///
    /// # TODO
    ///
    /// - Check no overwriting takes place?
    pub fn inline_node(self: &mut Self, offset: usize, dbg: DbgNode) {
        for entry in dbg.offsets {
            self.offsets.insert(entry.0 + offset, entry.1);
        }
        for child in dbg.children {
            self.append_node(child);
        }
    }

    /// Appends a node to **the children** of `self`.
    ///
    /// # Panics
    ///
    /// - if `self.children.len() ≤ 4`, meaning the cell has more than four cell-references.
    pub fn append_node(self: &mut Self, dbg: DbgNode) {
        assert!(self.children.len() <= 4);
        self.children.push(dbg)
    }
}

/// Multi-line display.
impl fmt::Display for DbgNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for entry in self.offsets.iter() {
            writeln!(f, "{}:{}", entry.0, entry.1)?
        }
        write!(f, "{} children", self.children.len())
    }
}

/// Stores data-offset position information for a bunch of cells.
///
/// Note that this type stores nothing about the sub-cells of each cell, as opposed to [`DbgNode`].
#[derive(Debug, Serialize, Deserialize)]
pub struct DbgInfo {
    /// Maps cell-hashes to data-offset position information.
    pub map: BTreeMap<String, OffsetPos>,
}
impl DbgInfo {
    /// Empty constructor.
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
    /// Constructor from a single cell.
    pub fn from(cell: &Cell, node: &DbgNode) -> Self {
        let mut info = Self::new();
        info.collect(&cell, &node);
        info
    }

    /// Number of cells registered.
    pub fn len(&self) -> usize {
        self.map.len()
    }
    /// True if no cells are registered.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Merges two selves.
    pub fn append(&mut self, other: &mut Self) {
        self.map.append(&mut other.map);
    }

    /// Inserts some data-offset info for a representation hash **if it is new**.
    pub fn insert(&mut self, key: UInt256, tree: OffsetPos) {
        self.map.entry(key.to_hex_string()).or_insert(tree);
    }
    /// Removes a hash/data-offset info binding.
    pub fn remove(&mut self, key: &UInt256) -> Option<OffsetPos> {
        self.map.remove(&key.to_hex_string())
    }
    /// Retrieves a hash/data-offset info binding.
    pub fn get(&self, key: &UInt256) -> Option<&OffsetPos> {
        self.map.get(&key.to_hex_string())
    }
    /// Accessor for the first entry in the cell hash map.
    pub fn first_entry(&self) -> Option<&OffsetPos> {
        self.map.iter().next().map(|k_v| k_v.1)
    }

    /// Adds a cell to the cell hash map, given its [`DbgNode`], **recursively**.
    ///
    /// # TODO
    ///
    /// Replace by [`Self::stackless_collect`]?
    fn collect(&mut self, cell: &Cell, dbg: &DbgNode) {
        let hash = cell.repr_hash().to_hex_string();
        // NB: existence of identical cells in a tree is normal.
        if !self.map.contains_key(&hash) {
            self.map.insert(hash, dbg.offsets.clone());
        }
        for i in 0..cell.references_count() {
            let child_cell = cell.reference(i).unwrap();
            let child_dbg = dbg.children[i].clone();
            self.collect(&child_cell, &child_dbg);
        }
    }

    /// Adds a cell to the cell hash map, given its [`DbgNode`], **recursively** (stackless).
    pub fn stackless_collect(&mut self, cell: &Cell, dbg: &DbgNode) {
        let mut stack: Vec<(Cell, &DbgNode)> = Vec::with_capacity(4);
        let mut hash: String;
        stack.push((cell.clone(), dbg));
        // We're cloning a cell here, but it's just an [`std::sync::Arc`] anyway.

        while let Some((cell, dbg)) = stack.pop() {
            hash = cell.repr_hash().to_hex_string();
            // NB: existence of identical cells in a tree is normal.
            if !self.map.contains_key(&hash) {
                self.map.insert(hash, dbg.offsets.clone());
            }
            for (subcell_idx, subcell) in cell.clone_references().into_iter().enumerate() {
                let subdbg = dbg
                    .children
                    .get(subcell_idx)
                    .expect("cell and debug node don't agree on the number of subcells");
                stack.push((subcell, subdbg))
            }
        }
    }
}
