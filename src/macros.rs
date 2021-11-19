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

//! This crate's macros.

#[macro_export]
#[doc(hidden)]
macro_rules! simple_commands_internal {

    // quantity of nothing is 0
    (@count ) => { 0u8 };

    // count quantity recursively
    (@count $_x:ident = $_y:ident; $($pname:ident = $parser:ident;)*) => {
        1u8 + $crate::simple_commands_internal!(
            @count $($pname = $parser;)*
        )
    };

    // Command with no parameters.
    (@resolve
        $command:ident
        => $code_head:expr $(, $code_tail:expr)*
    ) => {
        #[doc = concat!(
            "Parses a nullary `",
            stringify!($command),
            "` command.\n\nDefined as `",
            stringify!($code_head),
            $(" ", stringify!($code_tail), )*
            "`.",
        )]
        #[allow(non_snake_case)]
        pub fn $command(
            &mut self,
            par: &std::vec::Vec<&str>,
            destination: &mut T,
            pos: $crate::DbgPos
        ) -> $crate::CompileResult {
            par.assert_empty()?;
            destination.write_command(
                &[ $code_head $(, $code_tail)* ],
                $crate::debug::DbgNode::from(pos)
            )
        }
    };

    // parse command with any parameters
    (@resolve
        $command:ident $($pname:ident = $parser:ident);+
        => $code_head:expr $(, $code_tail:expr)*
    ) => {
        #[doc = concat!(
            "Parses a `",
            stringify!($command),
            $( " ", stringify!($pname) , )+
            "` command.\n\nDefined as `",
            stringify!($code_head),
            $(" ", stringify!($code_tail), )*
            "`.",
        )]
        #[allow(non_snake_case)]
        pub fn $command(
            &mut self,
            par: &std::vec::Vec<&str>,
            destination: &mut T,
            pos: $crate::DbgPos
        ) -> $crate::CompileResult {
            let n_params = $crate::simple_commands_internal!(
                @count $($pname = $parser;)*
            );
            par.assert_len(n_params as usize)?;
            let mut result: std::vec::Vec<u8> = vec![];
            let mut _parameters_i_:usize = 0;
            $(
                let $pname = $parser(par[_parameters_i_])
                    .parameter("arg ".to_string() + &_parameters_i_.to_string())?;
                _parameters_i_ += 1;
            )*
            result.push($code_head);
            $({ result.push($code_tail); })*
            destination.write_command(result.as_slice(), $crate::debug::DbgNode::from(pos))
        }
    };
}

/// Generates *simple* TVM commands, *i.e.* non-variadic commands.
///
/// Input: a sequence of command definitions of form
///
/// - command name (ident), then
/// - zero or more `;`-separated arguments of form
///     - parameter name (ident),
///     - `=`,
///     - parser name (ident),
///     with optional trailing `;`, then
/// - `=>` followed by
/// - a non-empty `,`-separated list of expressions.
#[macro_export]
macro_rules! simple_commands {
    // parse whole block of simple commands
    (
        $(
            $command:ident
            $( $pname:ident = $parser:ident );* $(;)?
            => $code_head:expr $( , $code_tail:expr )*
        )*
    ) => {
        $(
            $crate::simple_commands_internal!(
                @resolve $command $($pname = $parser);* => $code_head $(, $code_tail)*
            );
        )*

        #[doc = concat!(
            "Lists all the *simple* (non-variadic) commands.\n\n",
            "Simple commands are",
            $(
                "\n - [`",
                stringify!($command),
                $( " ", stringify!($pname), )*
                "`][Self::",
                stringify!($command),
                "] ≡ `",
                stringify!($code_head),
                $( " ", stringify!($code_tail), )*
                "`",
            )*
            "\n",
        )]
        pub fn enumerate_simple_commands() -> &'static [(&'static str, CompileHandler<T>)] {
            &[
                $( (stringify!($command), Engine::<T>::$command), )*
            ]
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! div_variant_internal {
    (@resolve $command:ident => $code: expr) => {
        impl<M: CommandBehaviourModifier> Div<M> {
            pub fn $command<T: Writer>(
                _engine: &mut Engine<T>,
                par: &Vec<&str>,
                destination: &mut T,
                pos: DbgPos,
            ) -> CompileResult {
                par.assert_len_in(0..=1)?;
                destination.write_command(
                    &M::modify({
                        if par.len() == 1 {
                            let v = $code | 0b00010000;
                            vec![0xA9, v, parse_const_u8_plus_one(par[0]).parameter("arg 0")?]
                        } else {
                            let v = $code & (!0b00010000);
                            vec![0xA9, v]
                        }
                    }),
                    DbgNode::from(pos),
                )
            }
        }
    };
}

#[macro_export]
macro_rules! div_variant {
    ($($command: ident => $code:expr)*) => {
        $(
            $crate::div_variant_internal!(
                @resolve $command => $code
            );
        )*
    };
}
