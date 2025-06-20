pub fn str_to_u8(origin_str: &str, target: &mut [u8]) -> isize {
    if origin_str.len() == 0 {-1}
    else{
        for i in 0..origin_str.len() {
            target[i] = origin_str.as_bytes()[i];
        }
        target[origin_str.len()+1]=b'\0';
        0
    }
}