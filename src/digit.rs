use core::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Sub};

#[cfg(feature = "ternary-string")]
use crate::Ternary;

/// ## Module: Balanced Ternary `Digit`
///
/// This module defines the `Digit` type for the balanced ternary numeral system,
/// along with its associated operations and functionality.
///
/// Represents a digit in the balanced ternary numeral system.
///
/// A digit can have one of three values:
/// - `Neg` (-1): Represents the value -1 in the balanced ternary system.
/// - `Zero` (0): Represents the value 0 in the balanced ternary system.
/// - `Pos` (+1): Represents the value +1 in the balanced ternary system.
///
/// Provides utility functions for converting to/from characters, integers, and negation.
///
/// ### Key Features
///
/// - **`Digit` Type**: Represents a digit in the balanced ternary numeral system.
///     - Possible values: `Neg` (-1), `Zero` (0), `Pos` (+1).
///     - Provides utility functions for converting between characters, integers, and other formats.
/// - **Arithmetic Operators**: Implements arithmetic operations for digits, including:
///     - Negation (`Neg`) and Bitwise Not (`Not`).
///     - Addition (`Add`) and Subtraction (`Sub`).
///     - Multiplication (`Mul`) and Division (`Div`), with safe handling of divisors (division by zero panics).
/// - **Logical Operators**: Supports bitwise logical operations (AND, OR, XOR) based on ternary logic rules.
/// - **Custom Methods**: Additional utility methods implementing balanced ternary logic principles.
///
/// ### Supported Use Cases
///
/// - Arithmetic in balanced ternary numeral systems.
/// - Logic operations in custom numeral systems.
/// - Conversion between balanced ternary representation and more common formats like integers and characters.
///
/// ## `Digit` type arithmetical and logical operations
///
/// - `Neg` and `Not` for `Digit`: Negates the digit value, adhering to balanced ternary rules.
/// - `Add<Digit>` for `Digit`: Adds two `Digit` values and returns a `Digit`.
/// - `Sub<Digit>` for `Digit`: Subtracts one `Digit` from another and returns a `Digit`.
/// - `Mul<Digit>` for `Digit`: Multiplies two `Digit` values and returns a `Digit`.
/// - `Div<Digit>` for `Digit`: Divides one `Digit` by another and returns a `Digit`. Division by zero panics.
///
/// ### Logical Operations for `Digit`
///
/// The `Digit` type supports bitwise logical operations, which are implemented according to logical rules applicable to balanced ternary digits.
///
/// ### Digits operations
/// 
/// ![Digit operations](https://raw.githubusercontent.com/Trehinos/balanced-ternary/refs/heads/master/digit-operations.png)
///
/// `/`, `*`, `&`, `|` and `^` should not be used with `Ternary::each_{with,zip}()`.
/// Instead, use these operators from `Ternary` directly.
///
/// Do so to `add` and `sub` ternaries, too.
/// # Optimization: `#[repr(i8)]` with value-matching discriminants
///
/// By assigning discriminants that match the logical values (`Neg = -1`,
/// `Zero = 0`, `Pos = 1`), conversions via `to_i8()` / `from_i8()` become
/// trivial casts instead of match trees. Since `to_i8()` sits in the
/// innermost loop of nearly every operation (`to_dec`, `each_zip`,
/// `balanced_carry`, add/sub), eliminating its match branch cascades
/// into speedups across the entire library.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum Digit {
    /// Represents -1
    Neg = -1,
    /// Represents 0
    Zero = 0,
    /// Represents +1
    Pos = 1,
}

impl Digit {

    /// Converts the `Digit` into a character representation.
    ///
    /// - Returns:
    ///     - `Θ` for `Digit::Neg`
    ///     - `0` for `Digit::Zero`
    ///     - `1` for `Digit::Pos`
    pub const fn to_char_theta(&self) -> char {
        match self {
            Digit::Neg => 'Θ',
            Digit::Zero => '0',
            Digit::Pos => '1',
        }
    }


    /// Converts the `Digit` into a character representation.
    ///
    /// - Returns:
    ///     - `Z` for `Digit::Neg`
    ///     - `0` for `Digit::Zero`
    ///     - `1` for `Digit::Pos`
    pub const fn to_char_z(&self) -> char {
        match self {
            Digit::Neg => 'Z',
            Digit::Zero => '0',
            Digit::Pos => '1',
        }
    }

    /// Converts the `Digit` into a character representation.
    ///
    /// - Returns:
    ///     - `T` for `Digit::Neg`
    ///     - `0` for `Digit::Zero`
    ///     - `1` for `Digit::Pos`
    pub const fn to_char_t(&self) -> char {
        match self {
            Digit::Neg => 'T',
            Digit::Zero => '0',
            Digit::Pos => '1',
        }
    }

    /// Creates a `Digit` from a character representation.
    ///
    /// - Accepts:
    ///     - `Θ` for `Digit::Neg`
    ///     - `0` for `Digit::Zero`
    ///     - `1` for `Digit::Pos`
    /// - Panics if the input character is invalid.
    pub const fn from_char_theta(c: char) -> Digit {
        match c { 
            'Θ' => Digit::Neg,
            '0' => Digit::Zero,
            '1' => Digit::Pos,
            _ => panic!("Invalid value. Expected 'Θ', '0', or '1'."),
        }
    }

    /// Creates a `Digit` from a character representation.
    ///
    /// - Accepts:
    ///     - `Z` for `Digit::Neg`
    ///     - `0` for `Digit::Zero`
    ///     - `1` for `Digit::Pos`
    /// - Panics if the input character is invalid.
    pub const fn from_char_z(c: char) -> Digit {
        match c {
            'Z' => Digit::Neg,
            '0' => Digit::Zero,
            '1' => Digit::Pos,
            _ => panic!("Invalid value. Expected 'Z', '0', or '1'."),
        }
    }

    /// Creates a `Digit` from a character representation.
    ///
    /// - Accepts:
    ///     - `T` for `Digit::Neg`
    ///     - `0` for `Digit::Zero`
    ///     - `1` for `Digit::Pos`
    /// - Panics if the input character is invalid.
    pub const fn from_char_t(c: char) -> Digit {
        match c {
            'T' => Digit::Neg,
            '0' => Digit::Zero,
            '1' => Digit::Pos,
            _ => panic!("Invalid value. Expected 'T', '0', or '1'."),
        }
    }
    
    /// Converts the `Digit` into its character representation.
    ///
    /// - Returns:
    ///     - `-` for `Digit::Neg`
    ///     - `0` for `Digit::Zero`
    ///     - `+` for `Digit::Pos`
    pub const fn to_char(&self) -> char {
        match self {
            Digit::Neg => '-',
            Digit::Zero => '0',
            Digit::Pos => '+',
        }
    }

    /// Creates a `Digit` from its character representation.
    ///
    /// - Accepts:
    ///     - `-` for `Digit::Neg`
    ///     - `0` for `Digit::Zero`
    ///     - `+` for `Digit::Pos`
    /// - Panics if the input character is invalid.
    pub const fn from_char(c: char) -> Digit {
        match c {
            '-' => Digit::Neg,
            '0' => Digit::Zero,
            '+' => Digit::Pos,
            _ => panic!("Invalid value. A Ternary must be either -, 0 or +."),
        }
    }

    /// Converts the `Digit` into its integer representation.
    ///
    /// - Returns:
    ///     - -1 for `Digit::Neg`
    ///     - 0 for `Digit::Zero`
    ///     - 1 for `Digit::Pos`
    ///
    /// # Optimization: zero-cost cast
    ///
    /// With `#[repr(i8)]` and discriminants `Neg = -1, Zero = 0, Pos = 1`,
    /// this is a direct reinterpretation of the enum's in-memory value —
    /// no branch, no lookup table.
    #[inline]
    pub const fn to_i8(&self) -> i8 {
        *self as i8
    }

    /// Creates a `Digit` from its integer representation.
    ///
    /// - Accepts:
    ///     - -1 for `Digit::Neg`
    ///     - 0 for `Digit::Zero`
    ///     - 1 for `Digit::Pos`
    /// - Panics if the input integer is invalid.
    #[inline]
    pub const fn from_i8(i: i8) -> Digit {
        match i {
            -1 => Digit::Neg,
            0 => Digit::Zero,
            1 => Digit::Pos,
            _ => panic!("Invalid value. A Ternary must be either -1, 0 or +1."),
        }
    }

    /// Returns the corresponding possible value of the current `Digit`.
    ///
    /// - Returns:
    ///     - `Digit::Neg` for `Digit::Neg`
    ///     - `Digit::Pos` for `Digit::Zero`
    ///     - `Digit::Pos` for `Digit::Pos`
    pub const fn possibly(self) -> Self {
        match self {
            Digit::Neg => Digit::Neg,
            Digit::Zero => Digit::Pos,
            Digit::Pos => Digit::Pos,
        }
    }

    /// Determines the condition of necessity for the current `Digit`.
    ///
    /// - Returns:
    ///     - `Digit::Neg` for `Digit::Neg`
    ///     - `Digit::Neg` for `Digit::Zero`
    ///     - `Digit::Pos` for `Digit::Pos`
    ///
    /// This method is used to calculate necessity as part
    /// of balanced ternary logic systems.
    pub const fn necessary(self) -> Self {
        match self {
            Digit::Neg => Digit::Neg,
            Digit::Zero => Digit::Neg,
            Digit::Pos => Digit::Pos,
        }
    }

    /// Determines the condition of contingency for the current `Digit`.
    ///
    /// - Returns:
    ///     - `Digit::Neg` for `Digit::Neg`
    ///     - `Digit::Pos` for `Digit::Zero`
    ///     - `Digit::Neg` for `Digit::Pos`
    ///
    /// This method represents contingency in balanced ternary logic,
    /// which defines the specific alternation of `Digit` values.
    pub const fn contingently(self) -> Self {
        match self {
            Digit::Neg => Digit::Neg,
            Digit::Zero => Digit::Pos,
            Digit::Pos => Digit::Neg,
        }
    }

    /// Returns the absolute positive value of the current `Digit`.
    ///
    /// - Returns:
    ///     - `Digit::Pos` for `Digit::Neg`
    ///     - `Digit::Zero` for `Digit::Zero`
    ///     - `Digit::Pos` for `Digit::Pos`
    pub const fn absolute_positive(self) -> Self {
        match self {
            Digit::Neg => Digit::Pos,
            Digit::Zero => Digit::Zero,
            Digit::Pos => Digit::Pos,
        }
    }

    /// Determines the strictly positive condition for the current `Digit`.
    ///
    /// - Returns:
    ///     - `Digit::Zero` for `Digit::Neg`
    ///     - `Digit::Zero` for `Digit::Zero`
    ///     - `Digit::Pos` for `Digit::Pos`
    ///
    /// This method is used to calculate strictly positive states
    /// in association with ternary logic.
    pub const fn positive(self) -> Self {
        match self {
            Digit::Neg => Digit::Zero,
            Digit::Zero => Digit::Zero,
            Digit::Pos => Digit::Pos,
        }
    }

    /// Determines the condition of non-negativity for the current `Digit`.
    ///
    /// - Returns:
    ///     - `Digit::Zero` for `Digit::Neg`
    ///     - `Digit::Pos` for `Digit::Zero`
    ///     - `Digit::Pos` for `Digit::Pos`
    ///
    /// This method is used to filter out negative conditions
    /// in computations with balanced ternary representations.
    pub const fn not_negative(self) -> Self {
        match self {
            Digit::Neg => Digit::Zero,
            Digit::Zero => Digit::Pos,
            Digit::Pos => Digit::Pos,
        }
    }

    /// Determines the condition of non-positivity for the current `Digit`.
    ///
    /// - Returns:
    ///     - `Digit::Neg` for `Digit::Neg`
    ///     - `Digit::Neg` for `Digit::Zero`
    ///     - `Digit::Zero` for `Digit::Pos`
    ///
    /// This method complements the `positive` condition and captures
    /// states that are not strictly positive.
    pub const fn not_positive(self) -> Self {
        match self {
            Digit::Neg => Digit::Neg,
            Digit::Zero => Digit::Neg,
            Digit::Pos => Digit::Zero,
        }
    }

    /// Determines the strictly negative condition for the current `Digit`.
    ///
    /// - Returns:
    ///     - `Digit::Neg` for `Digit::Neg`
    ///     - `Digit::Zero` for `Digit::Zero`
    ///     - `Digit::Zero` for `Digit::Pos`
    ///
    /// This method calculates strictly negative states
    /// in association with ternary logic.
    pub const fn negative(self) -> Self {
        match self {
            Digit::Neg => Digit::Neg,
            Digit::Zero => Digit::Zero,
            Digit::Pos => Digit::Zero,
        }
    }

    /// Returns the absolute negative value of the current `Digit`.
    ///
    /// - Returns:
    ///     - `Digit::Neg` for `Digit::Neg`
    ///     - `Digit::Zero` for `Digit::Zero`
    ///     - `Digit::Neg` for `Digit::Pos`
    pub const fn absolute_negative(self) -> Self {
        match self {
            Digit::Neg => Digit::Neg,
            Digit::Zero => Digit::Zero,
            Digit::Pos => Digit::Neg,
        }
    }

    /// Performs Kleene implication with the current `Digit` as `self` and another `Digit`.
    ///
    /// - `self`: The antecedent of the implication.
    /// - `other`: The consequent of the implication.
    ///
    /// - Returns:
    ///     - `Digit::Pos` when `self` is `Digit::Neg`.
    ///     - The positive condition of `other` when `self` is `Digit::Zero`.
    ///     - `other` when `self` is `Digit::Pos`.
    ///
    /// Implements Kleene ternary implication logic, which includes
    /// determining the logical result based on antecedent and consequent.
    pub const fn k3_imply(self, other: Self) -> Self {
        match self {
            Digit::Neg => Digit::Pos,
            Digit::Zero => other.positive(),
            Digit::Pos => other,
        }
    }

    /// Apply a ternary equivalence operation for the current `Digit` and another `Digit`.
    ///
    /// - `self`: The first operand of the equivalence operation.
    /// - `other`: The second operand of the equivalence operation.
    ///
    /// - Returns:
    ///     - The negation of `other` when `self` is `Digit::Neg`.
    ///     - `Digit::Zero` when `self` is `Digit::Zero`.
    ///     - `other` when `self` is `Digit::Pos`.
    ///
    /// This method implements a ternary logic equivalence, which captures the relationship between
    /// two balanced ternary `Digit`s based on their logical equivalence.
    /// # Optimization: k3_equiv = mul = a·b
    ///
    /// The truth table for k3_equiv is identical to multiplication:
    /// (−,−)→+, (−,0)→0, (−,+)→−, (0,·)→0, (+,·)→·
    /// This follows directly from Jones: equiv(a,b) = a·b.
    ///
    /// # Examples
    /// ```
    /// use balanced_ternary::Digit::{Neg, Pos, Zero};
    ///
    /// assert_eq!(Neg.k3_equiv(Neg), Pos);   // both false → true
    /// assert_eq!(Neg.k3_equiv(Zero), Zero); // false * unknown → unknown
    /// assert_eq!(Neg.k3_equiv(Pos), Neg);   // false * true → false
    /// assert_eq!(Zero.k3_equiv(Pos), Zero); // unknown * anything → unknown
    /// assert_eq!(Pos.k3_equiv(Neg), Neg);   // true * false → false
    /// assert_eq!(Pos.k3_equiv(Pos), Pos);   // both true → true
    /// ```
    pub const fn k3_equiv(self, other: Self) -> Self {
        // SAFETY: product of {-1,0,1} stays in {-1,0,1}.
        unsafe { core::mem::transmute::<i8, Digit>(self.to_i8() * other.to_i8()) }
    }

    /// Performs a ternary AND operation for the current `Digit` and another `Digit`.
    ///
    /// - `self`: The first operand of the AND operation.
    /// - `other`: The second operand of the AND operation.
    ///
    /// - Returns:
    ///     - `Digit::Neg` if `self` is `Digit::Neg` and `other` is not `Digit::Zero`.
    ///     - `Digit::Zero` if either `self` or `other` is `Digit::Zero`.
    ///     - `other` if `self` is `Digit::Pos`.
    ///
    /// This method implements Bochvar's internal three-valued logic in balanced ternary AND operation,
    /// which evaluates the logical conjunction of two `Digit`s in the ternary logic system.
    pub const fn bi3_and(self, other: Self) -> Self {
        match self {
            Digit::Neg => other.absolute_negative(),
            Digit::Zero => Digit::Zero,
            Digit::Pos => other,
        }
    }

    /// Performs a ternary OR operation for the current `Digit` and another `Digit`.
    ///
    /// - `self`: The first operand of the OR operation.
    /// - `other`: The second operand of the OR operation.
    ///
    /// - Returns:
    ///     - `other` if `self` is `Digit::Neg`.
    ///     - `Digit::Zero` if `self` is `Digit::Zero`.
    ///     - `Digit::Pos` if `self` is `Digit::Pos` and `other` is not `Digit::Zero`.
    ///
    /// This method implements Bochvar's three-valued internal ternary logic for the OR operation,
    /// determining the logical disjunction of two balanced ternary `Digit`s.
    pub const fn bi3_or(self, other: Self) -> Self {
        match self {
            Digit::Neg => other,
            Digit::Zero => Digit::Zero,
            Digit::Pos => other.absolute_positive(),
        }
    }

    /// Performs Bochvar's internal three-valued implication with the current `Digit` as `self`
    /// and another `Digit` as the consequent.
    ///
    /// - `self`: The antecedent of the implication.
    /// - `other`: The consequent of the implication.
    ///
    /// - Returns:
    ///     - `Digit::Zero` if `self` is `Digit::Neg` and `other` is `Digit::Zero`.
    ///     - `Digit::Pos` if `self` is `Digit::Neg` and `other` is not `Digit::Zero`.
    ///     - `Digit::Zero` if `self` is `Digit::Zero`.
    ///     - `other` if `self` is `Digit::Pos`.
    ///
    /// This method implements Bochvar's internal implication logic, which evaluates
    /// the logical consequence, between two balanced ternary `Digit`s in a manner
    /// consistent with three-valued logic principles.
    pub const fn bi3_imply(self, other: Self) -> Self {
        match self {
            Digit::Neg => other.absolute_positive(),
            Digit::Zero => Digit::Zero,
            Digit::Pos => other,
        }
    }

    /// Performs Łukasiewicz implication with the current `Digit` as `self` and another `Digit`.
    ///
    /// - `self`: The antecedent of the implication.
    /// - `other`: The consequent of the implication.
    ///
    /// - Returns:
    ///     - `Digit::Pos` when `self` is `Digit::Neg`.
    ///     - The non-negative condition of `other` when `self` is `Digit::Zero`.
    ///     - `other` when `self` is `Digit::Pos`.
    ///
    /// Implements Łukasiewicz ternary implication logic, which
    /// evaluates an alternative approach for implication compared to Kleene logic.
    pub const fn l3_imply(self, other: Self) -> Self {
        match self {
            Digit::Neg => Digit::Pos,
            Digit::Zero => other.not_negative(),
            Digit::Pos => other,
        }
    }

    /// Performs RM3 implication with the current `Digit` as `self` and another `Digit`.
    ///
    /// - `self`: The antecedent of the implication.
    /// - `other`: The consequent of the implication.
    ///
    /// - Returns:
    ///     - `Digit::Pos` when `self` is `Digit::Neg`.
    ///     - `other` when `self` is `Digit::Zero`.
    ///     - The necessary condition of `other` when `self` is `Digit::Pos`.
    ///
    /// Implements RM3 ternary implication logic, which defines a unique
    /// perspective for implication operations in balanced ternary systems.
    pub const fn rm3_imply(self, other: Self) -> Self {
        match self {
            Digit::Neg => Digit::Pos,
            Digit::Zero => other,
            Digit::Pos => other.necessary(),
        }
    }

    /// Performs the paraconsistent-logic implication with the current `Digit` as `self` and another `Digit`.
    ///
    /// - `self`: The antecedent of the implication.
    /// - `other`: The consequent of the implication.
    ///
    /// - Returns:
    ///     - `Digit::Pos` when `self` is `Digit::Neg`.
    ///     - `other` otherwise.
    pub const fn para_imply(self, other: Self) -> Self {
        match self {
            Digit::Neg => Digit::Pos,
            _ => other,
        }
    }

    /// Performs HT implication with the current `Digit` as `self` and another `Digit`.
    ///
    /// - `self`: The antecedent of the implication.
    /// - `other`: The consequent of the implication.
    ///
    /// - Returns:
    ///     - `Digit::Pos` when `self` is `Digit::Neg`.
    ///     - The possibility condition of `other` when `self` is `Digit::Zero`.
    ///     - `other` when `self` is `Digit::Pos`.
    ///
    /// This method computes HT ternary implication based on heuristic logic.
    pub const fn ht_imply(self, other: Self) -> Self {
        match self {
            Digit::Neg => Digit::Pos,
            Digit::Zero => other.possibly(),
            Digit::Pos => other,
        }
    }

    /// Performs HT logical negation of the current `Digit`.
    ///
    /// - Returns:
    ///     - `Digit::Pos` when `self` is `Digit::Neg`.
    ///     - `Digit::Neg` when `self` is `Digit::Zero` or `Digit::Pos`.
    ///
    /// This method evaluates the HT negation result using heuristic ternary logic.
    pub const fn ht_not(self) -> Self {
        match self {
            Digit::Neg => Digit::Pos,
            Digit::Zero => Digit::Neg,
            Digit::Pos => Digit::Neg,
        }
    }

    /// Converts the `Digit` to a `bool` in HT logic.
    ///
    /// - Returns:
    ///     - `true` when `self` is `Digit::Pos`.
    ///     - `false` when `self` is `Digit::Neg`.
    ///
    /// - Panics:
    ///     - Panics if `self` is `Digit::Zero`, as `Digit::Zero` cannot be directly
    ///       converted to a boolean value.
    ///       > To ensure `Pos` or `Neg` value, use one of :
    ///       > * [Digit::possibly]
    ///       > * [Digit::necessary]
    ///       > * [Digit::contingently]
    ///       > * [Digit::ht_not]
    ///
    pub const fn ht_bool(self) -> bool {
        match self {
            Digit::Neg => false,
            Digit::Zero => panic!(
                "Cannot convert a Digit::Zero to a bool. \
                 Use Digit::possibly()->to_bool() or Digit::necessary()->to_bool() instead."
            ),
            Digit::Pos => true,
        }
    }

    /// Performs Post's negation of the current `Digit`.
    ///
    /// - Returns:
    ///     - `Digit::Zero` when `self` is `Digit::Neg`.
    ///     - `Digit::Pos` when `self` is `Digit::Zero`.
    ///     - `Digit::Neg` when `self` is `Digit::Pos`.
    ///
    /// This method evaluates the negation based on Post's logic in ternary systems,
    /// which differs from standard negation logic.
    pub const fn post(self) -> Self {
        match self {
            Digit::Neg => Digit::Zero,
            Digit::Zero => Digit::Pos,
            Digit::Pos => Digit::Neg,
        }
    }

    /// Performs the inverse operation from the Post's negation of the current `Digit`.
    ///
    /// - Returns:
    ///     - `Digit::Pos` when `self` is `Digit::Neg`.
    ///     - `Digit::Neg` when `self` is `Digit::Zero`.
    ///     - `Digit::Zero` when `self` is `Digit::Pos`.
    pub const fn pre(self) -> Self {
        match self {
            Digit::Neg => Digit::Pos,
            Digit::Zero => Digit::Neg,
            Digit::Pos => Digit::Zero,
        }
    }

    /// This method maps this `Digit` value to its corresponding unbalanced ternary
    /// integer representation.
    ///
    /// - Returns:
    ///     - `0` for `Digit::Neg`.
    ///     - `1` for `Digit::Zero`.
    ///     - `2` for `Digit::Pos`.
    ///
    pub const fn to_unbalanced(self) -> u8 {
        match self {
            Digit::Neg => 0,
            Digit::Zero => 1,
            Digit::Pos => 2,
        }
    }

    /// Creates a `Digit` from an unbalanced ternary integer representation.
    ///
    /// # Arguments:
    /// - `u`: An unsigned 8-bit integer representing an unbalanced ternary value.
    ///
    /// # Returns:
    /// - `Digit::Neg` for `0`.
    /// - `Digit::Zero` for `1`.
    /// - `Digit::Pos` for `2`.
    ///
    /// # Panics:
    /// - Panics if the provided value is not `0`, `1`, or `2`, as these are the
    ///   only valid representations of unbalanced ternary values.
    pub const fn from_unbalanced(u: u8) -> Digit {
        match u {
            0 => Digit::Neg,
            1 => Digit::Zero,
            2 => Digit::Pos,
            _ => panic!("Invalid value. A unbalanced ternary value must be either 0, 1 or 2."),
        }
    }

    /// Tritwise **consensus**: returns the shared value when both trits agree, `Zero` otherwise.
    ///
    /// This extracts the positions where two ternary numbers are identical.
    /// Useful for range checks and filtering: only positions where both operands
    /// carry the same information survive.
    ///
    /// | `self` | `other` | result |
    /// |--------|---------|--------|
    /// | `+`    | `+`     | `+`    |
    /// | `-`    | `-`     | `-`    |
    /// | `0`    | `0`     | `0`    |
    /// | any    | differ  | `0`    |
    ///
    /// # Examples
    /// ```
    /// use balanced_ternary::Digit::{Neg, Pos, Zero};
    ///
    /// assert_eq!(Pos.consensus(Pos), Pos);
    /// assert_eq!(Neg.consensus(Neg), Neg);
    /// assert_eq!(Pos.consensus(Neg), Zero);
    /// assert_eq!(Zero.consensus(Pos), Zero);
    /// ```
    /// Branchless: `a · (a == b)`.
    /// When equal: `a * 1 = a`. When unequal: `a * 0 = 0`.
    ///
    /// Uses transmute instead of `from_i8` to avoid its `_ => panic!` arm.
    /// SAFETY: a·(a==b) for a,b ∈ {−1,0,1} yields a value in {−1,0,1}.
    pub const fn consensus(self, other: Self) -> Self {
        let a = self.to_i8();
        let b = other.to_i8();
        let v = a * ((a == b) as i8);
        // SAFETY: v ∈ {-1, 0, 1} — either a*1=a or a*0=0, both in {-1,0,1}.
        unsafe { core::mem::transmute::<i8, Digit>(v) }
    }

    /// Tritwise **accept-anything** (ANY): the non-zero trit wins; conflicting non-zero trits → `Zero`.
    ///
    /// `Zero` acts as a transparent pass-through. When both operands are non-zero
    /// and different, the conflict resolves to `Zero`. This lets two trytes with
    /// non-overlapping non-zero fields be merged losslessly:
    ///
    /// ```text
    /// 000---  ANY  +++000  =  +++---
    /// ```
    ///
    /// | `self` | `other` | result       |
    /// |--------|---------|--------------|
    /// | `0`    | any     | `other`      |
    /// | any    | `0`     | `self`       |
    /// | `+`    | `+`     | `+`          |
    /// | `-`    | `-`     | `-`          |
    /// | `+`    | `-`     | `0` conflict |
    /// | `-`    | `+`     | `0` conflict |
    ///
    /// # Examples
    /// ```
    /// use balanced_ternary::Digit::{Neg, Pos, Zero};
    ///
    /// assert_eq!(Zero.accept_anything(Pos), Pos);
    /// assert_eq!(Neg.accept_anything(Zero), Neg);
    /// assert_eq!(Pos.accept_anything(Pos), Pos);
    /// assert_eq!(Pos.accept_anything(Neg), Zero);
    /// ```
    /// # Optimization: Jones identity `sign(a + b)`
    ///
    /// All 9 cases reduce to the sign of the sum:
    /// - Both zero or conflict (±1 + ∓1 = 0) → 0.
    /// - One zero → the other (sign of ±1 = ±1).
    /// - Both equal → their shared sign (±1 + ±1 = ±2, sign = ±1).
    ///
    /// Eliminates the 5-arm match entirely.
    /// SAFETY: sign(a+b) for a,b ∈ {−1,0,1} is in {−1,0,1}.
    pub const fn accept_anything(self, other: Self) -> Self {
        let s = self.to_i8() + other.to_i8();
        let v = if s > 0 { 1i8 } else if s < 0 { -1i8 } else { 0i8 };
        // SAFETY: signum of sum of {-1,0,1} is in {-1,0,1}.
        unsafe { core::mem::transmute::<i8, Digit>(v) }
    }

    /// Decoder: returns `Pos` if `self == Neg`, otherwise `Neg`.
    ///
    /// Jones logic Function 2 / NTI ("Not True Indicator"). Useful for
    /// detecting the negative/false trit in a ternary word.
    ///
    /// ```
    /// use balanced_ternary::Digit::{Neg, Zero, Pos};
    ///
    /// assert_eq!(Neg.is_neg(),  Pos);
    /// assert_eq!(Zero.is_neg(), Neg);
    /// assert_eq!(Pos.is_neg(),  Neg);
    /// ```
    pub const fn is_neg(self) -> Self {
        if matches!(self, Self::Neg) { Self::Pos } else { Self::Neg }
    }

    /// Decoder: returns `Pos` if `self == Zero`, otherwise `Neg`.
    ///
    /// Jones logic Function K ("Is Unknown"). Detects the zero/unknown trit.
    ///
    /// ```
    /// use balanced_ternary::Digit::{Neg, Zero, Pos};
    ///
    /// assert_eq!(Neg.is_zero(),  Neg);
    /// assert_eq!(Zero.is_zero(), Pos);
    /// assert_eq!(Pos.is_zero(),  Neg);
    /// ```
    pub const fn is_zero(self) -> Self {
        if matches!(self, Self::Zero) { Self::Pos } else { Self::Neg }
    }

    /// Decoder: returns `Pos` if `self == Pos`, otherwise `Neg`.
    ///
    /// Jones logic Function 6 ("Is True"). Detects the positive/true trit.
    ///
    /// ```
    /// use balanced_ternary::Digit::{Neg, Zero, Pos};
    ///
    /// assert_eq!(Neg.is_pos(),  Neg);
    /// assert_eq!(Zero.is_pos(), Neg);
    /// assert_eq!(Pos.is_pos(),  Pos);
    /// ```
    pub const fn is_pos(self) -> Self {
        if matches!(self, Self::Pos) { Self::Pos } else { Self::Neg }
    }

    /// Clamp toward negative: `min(self, Zero)`.
    ///
    /// Jones logic Function C ("Clamp Down" / `a ∧ 0`).
    /// Returns `Neg` for negative input, `Zero` for zero or positive.
    ///
    /// ```
    /// use balanced_ternary::Digit::{Neg, Zero, Pos};
    ///
    /// assert_eq!(Neg.clamp_down(),  Neg);
    /// assert_eq!(Zero.clamp_down(), Zero);
    /// assert_eq!(Pos.clamp_down(),  Zero);
    /// ```
    pub const fn clamp_down(self) -> Self {
        if matches!(self, Self::Pos) { Self::Zero } else { self }
    }

    /// Clamp toward positive: `max(self, Zero)`.
    ///
    /// Jones logic Function R ("Clamp Up" / `a ∨ 0`).
    /// Returns `Pos` for positive input, `Zero` for zero or negative.
    ///
    /// ```
    /// use balanced_ternary::Digit::{Neg, Zero, Pos};
    ///
    /// assert_eq!(Neg.clamp_up(),  Zero);
    /// assert_eq!(Zero.clamp_up(), Zero);
    /// assert_eq!(Pos.clamp_up(),  Pos);
    /// ```
    pub const fn clamp_up(self) -> Self {
        if matches!(self, Self::Neg) { Self::Zero } else { self }
    }

    /// Ternary equality: returns `Pos` if `self == other`, `Neg` otherwise.
    ///
    /// Unlike Rust's `PartialEq` (which returns `bool`), this returns a `Digit`
    /// so it can be used as input to other ternary operations.
    ///
    /// ```
    /// use balanced_ternary::Digit::{Neg, Zero, Pos};
    ///
    /// assert_eq!(Pos.eq_digit(Pos),  Pos);
    /// assert_eq!(Neg.eq_digit(Neg),  Pos);
    /// assert_eq!(Pos.eq_digit(Neg),  Neg);
    /// assert_eq!(Zero.eq_digit(Pos), Neg);
    /// ```
    pub const fn eq_digit(self, other: Self) -> Self {
        if self.to_i8() == other.to_i8() { Self::Pos } else { Self::Neg }
    }

    /// Increments the `Digit` value and returns a `Ternary` result.
    ///
    /// - The rules for incrementing are based on ternary arithmetic:
    ///   - For `Digit::Neg`:
    ///     - Incrementing results in `Digit::Zero` (`Ternary::parse("0")`).
    ///   - For `Digit::Zero`:
    ///     - Incrementing results in `Digit::Pos` (`Ternary::parse("+")`).
    ///   - For `Digit::Pos`:
    ///     - Incrementing results in "overflow" (`Ternary::parse("+-")`).
    ///
    /// - Returns:
    ///   - A `Ternary` instance representing the result of the increment operation.
    #[cfg(feature = "ternary-string")]
    pub fn inc(self) -> Ternary {
        match self {
            Digit::Neg => Ternary::parse("0"),
            Digit::Zero => Ternary::parse("+"),
            Digit::Pos => Ternary::parse("+-"),
        }
    }

    /// Decrements the `Digit` value and returns a `Ternary` result.
    ///
    /// - The rules for decrementing are based on ternary arithmetic:
    ///   - For `Digit::Neg`:
    ///     - Decrementing results in "underflow" (`Ternary::parse("-+")`).
    ///   - For `Digit::Zero`:
    ///     - Decrementing results in `Digit::Neg` (`Ternary::parse("-")`).
    ///   - For `Digit::Pos`:
    ///     - Decrementing results in `Digit::Zero` (`Ternary::parse("0")`).
    ///
    /// - Returns:
    ///   - A `Ternary` instance representing the result of the decrement operation.
    #[cfg(feature = "ternary-string")]
    pub fn dec(self) -> Ternary {
        match self {
            Digit::Neg => Ternary::parse("-+"),
            Digit::Zero => Ternary::parse("-"),
            Digit::Pos => Ternary::parse("0"),
        }
    }
}

impl Neg for Digit {
    type Output = Self;

    /// Returns the negation of the `Digit`.
    ///
    /// - `Digit::Neg` becomes `Digit::Pos`
    /// - `Digit::Pos` becomes `Digit::Neg`
    /// - `Digit::Zero` remains `Digit::Zero`
    ///
    /// # Optimization: zero-cost transmute
    ///
    /// With `#[repr(i8)]` discriminants, negation is a single `neg` instruction.
    /// Uses transmute instead of `from_i8` to avoid the `_ => panic!` arm.
    /// SAFETY: −a for a ∈ {−1, 0, 1} is always in {−1, 0, 1}.
    #[inline]
    fn neg(self) -> Self::Output {
        // SAFETY: negation of {-1,0,1} stays in {-1,0,1}.
        unsafe { core::mem::transmute::<i8, Digit>(-self.to_i8()) }
    }
}

impl Not for Digit {
    type Output = Self;
    fn not(self) -> Self::Output {
        -self
    }
}

impl Add<Digit> for Digit {
    type Output = Digit;

    /// Adds two `Digit` values together and returns a `Digit` result.
    ///
    /// - Returns:
    ///   - A `Ternary` instance that holds the result of the addition.
    ///
    /// - Panics:
    ///   - This method does not panic under any circumstances.
    /// # Optimization: inline balanced arithmetic + transmute
    ///
    /// a + b for a,b ∈ {−1,0,1} gives s ∈ {−2,…,2}. Rebalance by ∓3:
    /// s=2 → −1, s=−2 → 1, else s unchanged — single pass, no match trees.
    /// SAFETY: rebalanced digit ∈ {−1, 0, 1}.
    fn add(self, other: Digit) -> Self::Output {
        let s = self.to_i8() + other.to_i8();
        let d = if s > 1 { s - 3 } else if s < -1 { s + 3 } else { s };
        // SAFETY: d ∈ {-1, 0, 1} after balanced rebalancing of s ∈ {-2..=2}.
        unsafe { core::mem::transmute::<i8, Digit>(d) }
    }
}

impl Sub<Digit> for Digit {
    type Output = Digit;

    /// Subtracts two `Digit` values and returns a `Digit` result.
    ///
    /// - Returns:
    ///   - A `Ternary` instance that holds the result of the subtraction.
    ///
    /// - Panics:
    ///   - This method does not panic under any circumstances.
    /// # Optimization: same inline balanced arithmetic as `add`
    fn sub(self, other: Digit) -> Self::Output {
        let s = self.to_i8() - other.to_i8();
        let d = if s > 1 { s - 3 } else if s < -1 { s + 3 } else { s };
        // SAFETY: d ∈ {-1, 0, 1} after balanced rebalancing of s ∈ {-2..=2}.
        unsafe { core::mem::transmute::<i8, Digit>(d) }
    }
}

impl Mul<Digit> for Digit {
    type Output = Digit;

    /// Multiplies two `Digit` values together and returns the product as a `Digit`.
    ///
    /// - The rules for multiplication in this implementation are straightforward:
    ///   - `Digit::Neg` multiplied by:
    ///     - `Digit::Neg` results in `Digit::Pos`.
    ///     - `Digit::Zero` results in `Digit::Zero`.
    ///     - `Digit::Pos` results in `Digit::Neg`.
    ///   - `Digit::Zero` multiplied by any `Digit` always results in `Digit::Zero`.
    ///   - `Digit::Pos` multiplied by:
    ///     - `Digit::Neg` results in `Digit::Neg`.
    ///     - `Digit::Zero` results in `Digit::Zero`.
    ///     - `Digit::Pos` results in `Digit::Pos`.
    ///
    /// - Returns:
    ///   - A `Digit` instance representing the result of the multiplication.
    /// # Optimization: direct i8 multiply + transmute
    ///
    /// Jones: mul is just sign-arithmetic. Product of {−1,0,1}² ⊆ {−1,0,1}.
    /// SAFETY: a·b for a,b ∈ {−1,0,1} is always in {−1,0,1}.
    fn mul(self, other: Digit) -> Self::Output {
        // SAFETY: product of {-1,0,1} stays in {-1,0,1}.
        unsafe { core::mem::transmute::<i8, Digit>(self.to_i8() * other.to_i8()) }
    }
}

impl Div<Digit> for Digit {
    type Output = Digit;

    /// Divides one `Digit` value by another and returns the result as a `Digit`.
    ///
    /// # Rules for division:
    /// - For `Digit::Neg`:
    ///   - Dividing `Digit::Neg` by `Digit::Neg` results in `Digit::Pos`.
    ///   - Dividing `Digit::Neg` by `Digit::Zero` will panic with "Cannot divide by zero."
    ///   - Dividing `Digit::Neg` by `Digit::Pos` results in `Digit::Neg`.
    /// - For `Digit::Zero`:
    ///   - Dividing `Digit::Zero` by `Digit::Neg` results in `Digit::Zero`.
    ///   - Dividing `Digit::Zero` by `Digit::Zero` will panic with "Cannot divide by zero."
    ///   - Dividing `Digit::Zero` by `Digit::Pos` results in `Digit::Zero`.
    /// - For `Digit::Pos`:
    ///   - Dividing `Digit::Pos` by `Digit::Neg` results in `Digit::Neg`.
    ///   - Dividing `Digit::Pos` by `Digit::Zero` will panic with "Cannot divide by zero."
    ///   - Dividing `Digit::Pos` by `Digit::Pos` results in `Digit::Pos`.
    ///
    /// # Returns:
    /// - A `Digit` value representing the result of the division.
    ///
    /// # Panics:
    /// - Panics with "Cannot divide by zero." if the `other` operand is `Digit::Zero`.
    fn div(self, other: Digit) -> Self::Output {
        if other == Digit::Zero {
            panic!("Cannot divide by zero.");
        }
        self * other
    }
}

impl BitAnd for Digit {
    type Output = Self;

    /// Performs a bitwise AND operation between two `Digit` values and returns the result.
    ///
    /// - The rules for the bitwise AND (`&`) operation are:
    ///   - If `self` is `Digit::Neg`, the result is always `Digit::Neg`.
    ///   - If `self` is `Digit::Zero`, the result depends on the value of `other`:
    ///     - `Digit::Neg` results in `Digit::Neg`.
    ///     - Otherwise, the result is `Digit::Zero`.
    ///   - If `self` is `Digit::Pos`, the result is simply `other`.
    ///
    /// # Returns:
    /// - A `Digit` value that is the result of the bitwise AND operation.
    ///
    /// # Examples:
    /// ```
    /// use balanced_ternary::Digit;
    /// use Digit::{Neg, Pos, Zero};
    ///
    /// assert_eq!(Neg & Pos, Neg);
    /// assert_eq!(Zero & Neg, Neg);
    /// assert_eq!(Zero & Pos, Zero);
    /// assert_eq!(Pos & Zero, Zero);
    /// ```
    /// Jones identity: AND = min(a, b).
    /// Selects between `self`/`other` directly — no `from_i8`, no panic path.
    /// Compiles to a single `cmp` + `cmov` on x86-64.
    fn bitand(self, other: Self) -> Self::Output {
        if self.to_i8() <= other.to_i8() { self } else { other }
    }
}

impl BitOr for Digit {
    type Output = Self;

    /// Performs a bitwise OR operation between two `Digit` values and returns the result.
    ///
    /// - The rules for the bitwise OR (`|`) operation are as follows:
    ///   - If `self` is `Digit::Neg`, the result is always the value of `other`.
    ///   - If `self` is `Digit::Zero`, the result depends on the value of `other`:
    ///     - `Digit::Pos` results in `Digit::Pos`.
    ///     - Otherwise, the result is `Digit::Zero`.
    ///   - If `self` is `Digit::Pos`, the result is always `Digit::Pos`.
    ///
    /// # Returns:
    /// - A `Digit` value that is the result of the bitwise OR operation.
    ///
    /// # Examples:
    /// ```
    /// use balanced_ternary::Digit;
    /// use Digit::{Neg, Pos, Zero};
    ///
    /// assert_eq!(Neg | Pos, Pos);
    /// assert_eq!(Zero | Neg, Zero);
    /// assert_eq!(Zero | Pos, Pos);
    /// assert_eq!(Pos | Zero, Pos);
    /// ```
    /// Jones identity: OR = max(a, b).
    /// Selects between `self`/`other` directly — no `from_i8`, no panic path.
    /// Compiles to a single `cmp` + `cmov` on x86-64.
    fn bitor(self, other: Self) -> Self::Output {
        if self.to_i8() >= other.to_i8() { self } else { other }
    }
}

impl BitXor for Digit {
    type Output = Self;

    /// Performs a bitwise XOR (exclusive OR) operation between two `Digit` values.
    ///
    /// - The rules for the bitwise XOR (`^`) operation are as follows:
    ///   - If `self` is `Digit::Neg`, the result is always the value of `rhs`.
    ///   - If `self` is `Digit::Zero`, the result is always `Digit::Zero`.
    ///   - If `self` is `Digit::Pos`, the result is the negation of `rhs`:
    ///     - `Digit::Neg` becomes `Digit::Pos`.
    ///     - `Digit::Zero` becomes `Digit::Zero`.
    ///     - `Digit::Pos` becomes `Digit::Neg`.
    ///
    /// # Returns:
    /// - A `Digit` value that is the result of the bitwise XOR operation.
    ///
    /// # Examples:
    /// ```
    /// use balanced_ternary::Digit;
    /// use Digit::{Neg, Pos, Zero};
    ///
    /// assert_eq!(Neg ^ Pos, Pos);
    /// assert_eq!(Zero ^ Neg, Zero);
    /// assert_eq!(Pos ^ Pos, Neg);
    /// ```
    /// Jones identity: XOR = −(a · b).
    ///
    /// Verified: same sign → product +1 → negate → −1 (false).
    ///           diff sign → product −1 → negate → +1 (true).
    ///           either zero → product 0 → negate → 0 (unknown).
    ///
    /// Uses transmute instead of `from_i8` to avoid its `_ => panic!` arm.
    /// SAFETY: −(a·b) for a,b ∈ {−1,0,1} always yields a value in {−1,0,1}.
    fn bitxor(self, rhs: Self) -> Self::Output {
        let v = -(self.to_i8() * rhs.to_i8());
        // SAFETY: product of {-1,0,1} is in {-1,0,1}; negation preserves that.
        unsafe { core::mem::transmute::<i8, Digit>(v) }
    }
}

#[cfg(test)]
mod tests {
    use super::Digit::{Neg, Pos, Zero};

    #[test]
    fn consensus_agreement() {
        assert_eq!(Pos.consensus(Pos), Pos);
        assert_eq!(Neg.consensus(Neg), Neg);
        assert_eq!(Zero.consensus(Zero), Zero);
    }

    #[test]
    fn consensus_disagreement_gives_zero() {
        assert_eq!(Pos.consensus(Neg), Zero);
        assert_eq!(Neg.consensus(Pos), Zero);
        assert_eq!(Pos.consensus(Zero), Zero);
        assert_eq!(Zero.consensus(Pos), Zero);
        assert_eq!(Neg.consensus(Zero), Zero);
        assert_eq!(Zero.consensus(Neg), Zero);
    }

    #[test]
    fn accept_anything_zero_is_transparent() {
        assert_eq!(Zero.accept_anything(Pos), Pos);
        assert_eq!(Zero.accept_anything(Neg), Neg);
        assert_eq!(Zero.accept_anything(Zero), Zero);
        assert_eq!(Pos.accept_anything(Zero), Pos);
        assert_eq!(Neg.accept_anything(Zero), Neg);
    }

    #[test]
    fn accept_anything_agreement() {
        assert_eq!(Pos.accept_anything(Pos), Pos);
        assert_eq!(Neg.accept_anything(Neg), Neg);
    }

    #[test]
    fn accept_anything_conflict_gives_zero() {
        assert_eq!(Pos.accept_anything(Neg), Zero);
        assert_eq!(Neg.accept_anything(Pos), Zero);
    }

    // Where consensus gives a non-zero result, accept_anything must agree.
    #[test]
    fn consensus_implies_accept_anything() {
        for a in [Neg, Zero, Pos] {
            for b in [Neg, Zero, Pos] {
                let con = a.consensus(b);
                let any = a.accept_anything(b);
                if con != Zero {
                    assert_eq!(any, con, "a={a:?} b={b:?}");
                }
            }
        }
    }
}
