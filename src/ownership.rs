// Ownership and borrow checking types for psudo
// Mirrors Rust's ownership model: Owned, Borrowed(&), MutBorrowed(&mut)

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::parser::DataTypes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lifetime(pub usize);

impl Lifetime {
    pub fn new() -> Self {
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
        Lifetime(COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

impl Default for Lifetime {
    fn default() -> Self {
        Lifetime::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
#[derive(Default)]
pub enum Ownership {
    /// Fully owned value - can be moved, dropped, or borrowed
    #[default]
    Owned,
    /// Immutable borrow - read-only access, multiple allowed
    Borrowed(Lifetime),
    /// Mutable borrow - exclusive write access, only one allowed
    MutBorrowed(Lifetime),
    /// Static lifetime - never dropped, always valid
    Static,
}


impl Ownership {
    pub fn is_owned(&self) -> bool {
        matches!(self, Ownership::Owned)
    }

    pub fn is_borrowed(&self) -> bool {
        matches!(self, Ownership::Borrowed(_))
    }

    pub fn is_mut_borrowed(&self) -> bool {
        matches!(self, Ownership::MutBorrowed(_))
    }

    pub fn lifetime(&self) -> Option<Lifetime> {
        match self {
            Ownership::Borrowed(l) | Ownership::MutBorrowed(l) => Some(*l),
            _ => None,
        }
    }
}

/// A value with ownership tracking
#[derive(Debug, Clone)]
pub struct OwnedValue<T> {
    pub inner: T,
    pub ownership: Ownership,
    pub moved: bool,
}

impl<T> OwnedValue<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            ownership: Ownership::Owned,
            moved: false,
        }
    }

    pub fn new_static(inner: T) -> Self {
        Self {
            inner,
            ownership: Ownership::Static,
            moved: false,
        }
    }

    pub fn borrow(&mut self) -> Result<BorrowRef<'_, T>, BorrowError> {
        match self.ownership {
            Ownership::Owned => {
                let lt = Lifetime::new();
                self.ownership = Ownership::Borrowed(lt);
                Ok(BorrowRef {
                    value: self,
                    lifetime: lt,
                    mutable: false,
                })
            }
            Ownership::Borrowed(_) => {
                let lt = Lifetime::new();
                self.ownership = Ownership::Borrowed(lt);
                Ok(BorrowRef {
                    value: self,
                    lifetime: lt,
                    mutable: false,
                })
            }
            Ownership::MutBorrowed(_) => Err(BorrowError::AlreadyMutBorrowed),
            Ownership::Static => {
                let lt = Lifetime::new();
                Ok(BorrowRef {
                    value: self,
                    lifetime: lt,
                    mutable: false,
                })
            }
        }
    }

    pub fn borrow_mut(&mut self) -> Result<BorrowRefMut<'_, T>, BorrowError> {
        match self.ownership {
            Ownership::Owned => {
                let lt = Lifetime::new();
                self.ownership = Ownership::MutBorrowed(lt);
                Ok(BorrowRefMut {
                    value: self,
                    lifetime: lt,
                })
            }
            Ownership::Borrowed(_) => Err(BorrowError::AlreadyBorrowed),
            Ownership::MutBorrowed(_) => Err(BorrowError::AlreadyMutBorrowed),
            Ownership::Static => Err(BorrowError::StaticImmutable),
        }
    }

    pub fn release_borrow(&mut self, lt: Lifetime, mutable: bool) {
        if mutable {
            if matches!(self.ownership, Ownership::MutBorrowed(l) if l == lt) {
                self.ownership = Ownership::Owned;
            }
        } else {
            // For immutable borrows, we don't track count - simplify for now
            if matches!(self.ownership, Ownership::Borrowed(l) if l == lt) {
                self.ownership = Ownership::Owned;
            }
        }
    }

    pub fn take(&mut self) -> Result<T, MoveError> {
        if self.moved {
            return Err(MoveError::UseAfterMove);
        }
        if self.ownership != Ownership::Owned && self.ownership != Ownership::Static {
            return Err(MoveError::BorrowedWhenMoving);
        }
        self.moved = true;
        // Can't actually move inner due to ownership, so we need a different approach
        Err(MoveError::NotImplemented)
    }
}

#[derive(Debug)]
pub enum BorrowError {
    AlreadyBorrowed,
    AlreadyMutBorrowed,
    StaticImmutable,
}

#[derive(Debug)]
pub enum MoveError {
    UseAfterMove,
    BorrowedWhenMoving,
    NotImplemented,
}

impl fmt::Display for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorrowError::AlreadyBorrowed => write!(f, "cannot borrow mutably: already borrowed"),
            BorrowError::AlreadyMutBorrowed => write!(f, "cannot borrow: already mutably borrowed"),
            BorrowError::StaticImmutable => write!(f, "cannot mutably borrow static value"),
        }
    }
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveError::UseAfterMove => write!(f, "use of moved value"),
            MoveError::BorrowedWhenMoving => write!(f, "cannot move while borrowed"),
            MoveError::NotImplemented => write!(f, "move not implemented"),
        }
    }
}

pub struct BorrowRef<'a, T> {
    value: &'a mut OwnedValue<T>,
    lifetime: Lifetime,
    mutable: bool,
}

impl<'a, T> Drop for BorrowRef<'a, T> {
    fn drop(&mut self) {
        self.value.release_borrow(self.lifetime, self.mutable);
    }
}

impl<'a, T> std::ops::Deref for BorrowRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value.inner
    }
}

pub struct BorrowRefMut<'a, T> {
    value: &'a mut OwnedValue<T>,
    lifetime: Lifetime,
}

impl<'a, T> Drop for BorrowRefMut<'a, T> {
    fn drop(&mut self) {
        self.value.release_borrow(self.lifetime, true);
    }
}

impl<'a, T> std::ops::Deref for BorrowRefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value.inner
    }
}

impl<'a, T> std::ops::DerefMut for BorrowRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value.inner
    }
}

/// Borrow checker for interpreter - tracks loans per variable
#[derive(Debug, Default)]
pub struct BorrowChecker {
    /// Loans: variable name -> list of active loans
    loans: HashMap<String, Vec<Loan>>,
    /// Moved variables
    moved: HashMap<String, Lifetime>,
    /// Next lifetime ID
    next_lifetime: usize,
}

#[derive(Debug, Clone)]
pub struct Loan {
    pub lifetime: Lifetime,
    pub mutable: bool,
    pub location: String, // source location for error messages
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enter_scope(&mut self) {
        // Scope tracking could be added here
    }

    pub fn exit_scope(&mut self, scope_vars: &[String]) {
        for var in scope_vars {
            self.loans.remove(var);
            self.moved.remove(var);
        }
    }

    pub fn borrow(
        &mut self,
        var: &str,
        mutable: bool,
        location: String,
    ) -> Result<Lifetime, BorrowError> {
        let lt = Lifetime(self.next_lifetime);
        self.next_lifetime += 1;

        let loans = self.loans.entry(var.to_string()).or_default();

        if mutable {
            // Mutable borrow: no other loans allowed
            if !loans.is_empty() {
                return Err(BorrowError::AlreadyBorrowed);
            }
            loans.push(Loan {
                lifetime: lt,
                mutable: true,
                location,
            });
        } else {
            // Immutable borrow: no mutable loans allowed
            if loans.iter().any(|l| l.mutable) {
                return Err(BorrowError::AlreadyMutBorrowed);
            }
            loans.push(Loan {
                lifetime: lt,
                mutable: false,
                location,
            });
        }

        Ok(lt)
    }

    pub fn release(&mut self, var: &str, lt: Lifetime) {
        if let Some(loans) = self.loans.get_mut(var) {
            loans.retain(|l| l.lifetime != lt);
            if loans.is_empty() {
                self.loans.remove(var);
            }
        }
    }

    pub fn move_var(&mut self, var: &str, _location: String) -> Result<(), MoveError> {
        // Check if borrowed
        if let Some(loans) = self.loans.get(var) {
            if !loans.is_empty() {
                return Err(MoveError::BorrowedWhenMoving);
            }
        }

        // Check if already moved
        if self.moved.contains_key(var) {
            return Err(MoveError::UseAfterMove);
        }

        self.moved
            .insert(var.to_string(), Lifetime(self.next_lifetime));
        self.next_lifetime += 1;
        Ok(())
    }

    pub fn use_var(&mut self, var: &str, _location: String) -> Result<(), MoveError> {
        if self.moved.contains_key(var) {
            return Err(MoveError::UseAfterMove);
        }
        Ok(())
    }

    pub fn is_moved(&self, var: &str) -> bool {
        self.moved.contains_key(var)
    }
    pub fn has_active_loans(&self, var: &str) -> bool {
        self.loans.get(var).map(|l| !l.is_empty()).unwrap_or(false)
    }

    pub fn release_all(&mut self) {
        self.loans.clear();
        self.moved.clear();
    }
}

#[derive(Debug, Clone)]
pub struct CheckedEnvironment {
    pub vars: HashMap<String, DataTypes>,
    pub parent: Option<Rc<RefCell<CheckedEnvironment>>>,
    pub borrow_checker: Rc<RefCell<BorrowChecker>>,
}

impl CheckedEnvironment {
    pub fn new(borrow_checker: Rc<RefCell<BorrowChecker>>) -> Self {
        Self {
            vars: HashMap::new(),
            parent: None,
            borrow_checker,
        }
    }

    pub fn with_parent(
        vars: HashMap<String, DataTypes>,
        parent: Rc<RefCell<CheckedEnvironment>>,
    ) -> Self {
        let borrow_checker = parent.borrow().borrow_checker.clone();
        Self {
            vars,
            parent: Some(parent),
            borrow_checker,
        }
    }

    pub fn define(&mut self, name: String, val: DataTypes) {
        self.vars.insert(name, val);
    }

    pub fn get(&self, name: &str) -> Option<DataTypes> {
        if let Some(v) = self.vars.get(name) {
            Some(v.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn declare(&mut self, name: String, val: DataTypes) -> Result<(), MoveError> {
        // Don't check move for new declarations - just insert
        self.vars.insert(name, val);
        Ok(())
    }

    pub fn assign(&mut self, name: String, val: DataTypes) -> Result<(), MoveError> {
        self.check_use(&name, "assign".to_string())?;
        self.vars.insert(name, val);
        Ok(())
    }

    pub fn push_scope(&mut self) {
        // For now, just create a new scope with same borrow checker
        // In a full implementation, this would track scope depth
    }

    pub fn pop_scope(&mut self) {
        // For now, just release all borrows
        // In a full implementation, this would release scope-specific borrows
        self.borrow_checker.borrow_mut().release_all();
    }

    pub fn check_move(&mut self, name: &str, location: String) -> Result<(), MoveError> {
        self.borrow_checker.borrow_mut().move_var(name, location)
    }

    pub fn check_use(&mut self, name: &str, location: String) -> Result<(), MoveError> {
        self.borrow_checker.borrow_mut().use_var(name, location)
    }

    pub fn borrow(
        &mut self,
        name: &str,
        mutable: bool,
        location: String,
    ) -> Result<Lifetime, BorrowError> {
        self.borrow_checker
            .borrow_mut()
            .borrow(name, mutable, location)
    }

    pub fn release_borrow(&mut self, name: &str, lt: Lifetime) {
        self.borrow_checker.borrow_mut().release(name, lt)
    }
}
