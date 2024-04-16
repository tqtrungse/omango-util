/// [ANY] wraps `std::any::Any` with useful functions to cast to
/// the original type. 
/// 
/// 
/// # Examples
/// 
/// ```
/// use omango_util::any::ANY;
///
/// struct Dummy {}
///     
/// impl Dummy {
///     #[inline(always)]
///     fn new() -> Self {
///         Self{}
///     }
///         
///     fn get_type(&self) -> String {
///        String::from("dummy")
///     }
///}
///     
///impl ANY for Dummy {
///     #[inline(always)]
///      fn as_mut(&mut self) -> &mut dyn std::any::Any { 
///         self
///      }
///     
///      #[inline(always)]
///      fn as_ref(&self) -> &dyn std::any::Any {
///         self
///      }
///}
///     
///fn parse(d: &dyn ANY) {
///    let r = d.as_ref();
///    if r.type_id() == std::any::TypeId::of::<Dummy>() {
///         assert_eq!(
///             r.downcast_ref::<Dummy>().unwrap().get_type(), 
///             String::from("dummy")
///         );
///    }
///}
///
///fn test() {
///     let d = Box::new(Dummy::new());
///     parse(d.as_ref());
///}
/// ```
pub trait ANY {
    /// [as_mut] returns mutable [std::any::Any] of raw type.
    fn as_mut(&mut self) -> &mut dyn std::any::Any;

    /// [as_ref] returns reference [std::any::Any] of raw type.
    fn as_ref(&self) -> &dyn std::any::Any;
}

mod test {
    #[test]
    fn test() {
        struct Dummy {}
        
        impl Dummy {
            #[inline(always)]
            fn new() -> Self {
                Self{}
            }
            
            fn get_type(&self) -> String {
                String::from("dummy")
            }
        }
        
        impl crate::any::ANY for Dummy {
            #[inline(always)]
            fn as_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        
            #[inline(always)]
            fn as_ref(&self) -> &dyn std::any::Any {
                self
            }
        }
        
        fn parse(d: &dyn crate::any::ANY) {
            let r = d.as_ref();
            if r.type_id() == std::any::TypeId::of::<Dummy>() {
                assert_eq!(
                    r.downcast_ref::<Dummy>().unwrap().get_type(), 
                    String::from("dummy")
                );
            }
            assert!(r.downcast_ref::<i32>().is_none());
        }
        
        let d = Box::new(Dummy::new());
        parse(d.as_ref());
    }
}