fn ret_string() -> String {
    String::from("  A String object  ")
}

fn choose_str<'a1:'b, 'a2:'b,'b>(s1: &'a1 str, s2: &'a2 str, select_s1: bool) -> &'b str {
    if select_s1 { s1 } else { s2 }
}

enum OOR<'a>{
    Owned(String),
    Borrowed(&'a str),
}

impl std::ops::Deref for OOR<'_>{
    type Target=str;

    fn deref(&self) -> &Self::Target {
        match self{
            OOR::Owned(s) => s,
            OOR::Borrowed(s)=> *s,
        }
    }
}

impl std::ops::DerefMut for OOR<'_>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            OOR::Owned(s) => s.as_mut_str(),
            OOR::Borrowed(s) => {
                *self = OOR::Owned(s.to_owned());
                return self
            },
        }
    }
}
fn main() {
    let mut s:String = ret_string();
    s=s.trim().to_string();
    assert_eq!(s, "A String object");

    // Check Deref for both variants of OOR
    let s1 = OOR::Owned(String::from("  Hello, world.  "));
    assert_eq!(s1.trim(), "Hello, world.");
    let mut s2 = OOR::Borrowed("  Hello, world!  ");
    assert_eq!(s2.trim(), "Hello, world!");

    // Check choose
    let s = choose_str(&s1, &s2, true);
    assert_eq!(s.trim(), "Hello, world.");
    let s = choose_str(&s1, &s2, false);
    assert_eq!(s.trim(), "Hello, world!");

    // Check DerefMut, a borrowed string should become owned
    assert!(matches!(s1, OOR::Owned(_)));
    assert!(matches!(s2, OOR::Borrowed(_)));
    unsafe {
        for c in s2.as_bytes_mut() {
            if *c == b'!' {
                *c = b'?';
            }
        }
    }
    assert!(matches!(s2, OOR::Owned(_)));
    assert_eq!(s2.trim(), "Hello, world?");
}