//! `Option` extensions

/// Takes two `Option`s and returns a sinle
/// `Option` wrapping both inner values.
///
/// If either `Option` is `None`, the result
/// will also be `None`.
///
/// # Example
///
/// ```ignore
/// let a = Some("a");
/// let b = Some(1);
/// let c = lift2(a, b);
///
/// assert_eq!(Some(("a", 1)), c);
/// ```
fn lift2<A, B>(a: Option<A>, b: Option<B>) -> Option<(A, B)> {
    a.and_then(|ai| b.map(|bi| (ai, bi)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lift() {
        assert_eq!(Some(("a", 1)), lift2(Some("a"), Some(1)));
        assert_eq!(None, lift2::<&str, i32>(None, Some(1)));
        assert_eq!(None, lift2::<&str, i32>(Some("a"), None));
        assert_eq!(None, lift2::<&str, i32>(None, None));
    }
}
