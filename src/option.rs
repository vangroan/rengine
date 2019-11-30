//! `Option` extensions

/// Takes two `Option`s and returns a single
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
pub fn lift2<A, B>(a: Option<A>, b: Option<B>) -> Option<(A, B)> {
    a.and_then(|ai| b.map(|bi| (ai, bi)))
}

pub fn lift3<A, B, C>(a: Option<A>, b: Option<B>, c: Option<C>) -> Option<(A, B, C)> {
    a.and_then(|ai| b.and_then(|bi| c.map(|ci| (ai, bi, ci))))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lift2() {
        assert_eq!(Some(("a", 1)), lift2(Some("a"), Some(1)));
        assert_eq!(None, lift2::<&str, i32>(None, Some(1)));
        assert_eq!(None, lift2::<&str, i32>(Some("a"), None));
        assert_eq!(None, lift2::<&str, i32>(None, None));
    }

    #[test]
    fn test_lift3() {
        assert_eq!(Some(("a", 1, true)), lift3(Some("a"), Some(1), Some(true)));
        assert_eq!(None, lift3::<&str, i32, bool>(None, Some(1), Some(true)));
        assert_eq!(None, lift3::<&str, i32, bool>(Some("a"), None, Some(true)));
        assert_eq!(None, lift3::<&str, i32, bool>(None, None, Some(true)));
        assert_eq!(None, lift3::<&str, i32, bool>(None, Some(1), None));
        assert_eq!(None, lift3::<&str, i32, bool>(Some("a"), Some(1), None));
    }
}
