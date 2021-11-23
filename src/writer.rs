/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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

//! [`Writer`] trait and codepages (only [`CodePage0`] for now).

use ton_types::{BuilderData, SliceData};

use crate::{
    debug::{DbgNode, DbgPos},
    OperationError,
};

/// Writes the result of compiling some code.
pub trait Writer: 'static {
    /// Constructor.
    fn new() -> Self;
    /// Writes a command with no additional data.
    fn write_command(&mut self, command: &[u8], dbg: DbgNode) -> Result<(), OperationError>;
    /// Writes a command and some bits to the data (bitstring) of a cell.
    fn write_command_bitstring(
        &mut self,
        command: &[u8],
        bits: usize,
        dbg: DbgNode,
    ) -> Result<(), OperationError>;
    /// Writes a composite command.
    fn write_composite_command(
        &mut self,
        code: &[u8],
        reference: BuilderData,
        pos: DbgPos,
        dbg: DbgNode,
    ) -> Result<(), OperationError>;
    /// Finalizes the writing process.
    fn finalize(self) -> (BuilderData, DbgNode);
}

/// First TON codepage.
///
/// A codepage gives semantics to opcodes. More codepages may be added in the future and modify
/// existing opcodes (semantics or encoding) or add new ones.
///
/// # Invariants
///
/// - `self.cells.len() == self.dbg.len()` (should just have one vector);
/// - all cells in [`self.cells`] have at least one free cell-ref (needed for [`Self::finalize`]).
pub(crate) struct CodePage0 {
    /// List of cells being constructed.
    cells: Vec<BuilderData>,
    /// Debug information (source-position and such) of cells.
    ///
    /// Must have the same length as [`self.cells`].
    dbg: Vec<DbgNode>,
}

impl Writer for CodePage0 {
    /// Constructor.
    fn new() -> Self {
        Self {
            cells: vec![BuilderData::new()],
            dbg: vec![DbgNode::new()],
        }
    }

    /// Writes a command.
    fn write_command(&mut self, command: &[u8], dbg: DbgNode) -> Result<(), OperationError> {
        self.write_command_bitstring(command, command.len() * 8, dbg)
    }

    /// Writes a command to a cell's data.
    ///
    /// - to the last cell in the list of cell if any,
    /// - to a new cell otherwise.
    fn write_command_bitstring(
        &mut self,
        command: &[u8],
        bits: usize,
        dbg: DbgNode,
    ) -> Result<(), OperationError> {
        // #optim rewrite to an `if let Some(_)`
        if !self.cells.is_empty() {
            let offset = self.cells.last().unwrap().bits_used();
            if self
                .cells
                .last_mut()
                .unwrap()
                .append_raw(command, bits)
                .is_ok()
            {
                self.dbg.last_mut().unwrap().inline_node(offset, dbg);
                return Ok(());
            }
        }
        let mut code = BuilderData::new();
        if code.append_raw(command, bits).is_ok() {
            self.cells.push(code);
            self.dbg.push(dbg);
            return Ok(());
        }
        Err(OperationError::NotFitInSlice)
    }

    /// Writes a composit command.
    ///
    /// - to the last cell in the list of cell if any,
    /// - to a new cell otherwise.
    fn write_composite_command(
        &mut self,
        command: &[u8],
        reference: BuilderData,
        pos: DbgPos,
        dbg: DbgNode,
    ) -> Result<(), OperationError> {
        // #optim rewrite to an `if let Some(_)`
        if !self.cells.is_empty() {
            let mut last = self.cells.last().unwrap().clone();
            let offset = last.bits_used();
            if last.references_free() > 1 // one cell remains reserved for finalization
                && last.append_raw(command, command.len() * 8).is_ok()
                && last.checked_append_reference(reference.clone().into_cell().map_err(|_| OperationError::NotFitInSlice)?).is_ok()
            {
                *self.cells.last_mut().unwrap() = last;

                let node = self.dbg.last_mut().unwrap();
                node.append(offset, pos);
                node.append_node(dbg);
                return Ok(());
            }
        }

        let mut code = BuilderData::new();
        let cell = reference
            .into_cell()
            .map_err(|_| OperationError::NotFitInSlice)?;
        if code.append_raw(command, command.len() * 8).is_ok()
            && code.checked_append_reference(cell).is_ok()
        {
            self.cells.push(code);

            let mut node = DbgNode::new();
            node.append(0, pos);
            node.append_node(dbg);
            self.dbg.push(node);

            return Ok(());
        }
        Err(OperationError::NotFitInSlice)
    }

    /// Finalizes the writer.
    ///
    /// Say the list of cells in `self` is `c_0, c_1, ..., c_n`. Then this function adds `c_i` as a
    /// *"reference"* of `c_i-1` for `i > 1`, and yields `c_0`. It might not be added as an actual
    /// reference though, if possible `c_i` will be inlined in `c_i-1`; *possible* here means that
    /// `c_i-1` has enough free cell-refs to add `c_i`'s cell refs.
    fn finalize(mut self) -> (BuilderData, DbgNode) {
        let mut cursor = self.cells.pop().expect("cells can't be empty");
        let mut dbg = self.dbg.pop().expect("dbgs can't be empty");
        // #optim rewrite as `while let Some(_)`
        while !self.cells.is_empty() {
            let mut destination = self.cells.pop().expect("vector is not empty");
            let offset = destination.bits_used();
            let slice = SliceData::from(
                cursor
                    .clone()
                    .into_cell()
                    .expect("failure while convert BuilderData to cell"),
            );
            let mut next = self.dbg.pop().expect("dbg vector is not empty");

            // Try to inline `cursor` into `destination`.
            //
            // Only possible if the `destination` has enough free ref-cells to store `cursor`'s
            // ref-cells.
            if destination.references_free() >= cursor.references_used()
                && destination
                    .checked_append_references_and_data(&slice)
                    .is_ok()
            {
                next.inline_node(offset, dbg);
            } else {
                // Otherwise just add `cursor` to `destination` as a reference.
                //
                // Only legal if `cursor` has at least one free ref-cell.
                destination.append_reference_cell(
                    cursor
                        .into_cell()
                        .expect("failure while converting `BuilderData` to `Cell`"),
                );
                next.append_node(dbg);
            }
            cursor = destination;
            dbg = next;
        }
        (cursor, dbg)
    }
}
