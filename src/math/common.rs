pub fn float_equal(a: f32, b: f32) -> bool {
    let EPSILON = 0.00001;

    println!("float_equal: a = {}, b = {}", a, b);
    if (a - b).abs() < EPSILON {
        return true;
    }
    false
}
