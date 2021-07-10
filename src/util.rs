pub unsafe fn raw_cast<'a, T>(buffer: &'a [u8]) -> Option<&'a T>
where
    T: Sized
{
    let target_size = core::mem::size_of::<T>();
    if target_size > buffer.len() {
        return None;
    }

    let raw_ptr = buffer as *const [u8] as *const ();
    let raw_ptr: *const T = core::mem::transmute::<*const (), *const T>(raw_ptr);

    Some(& *raw_ptr)
}
