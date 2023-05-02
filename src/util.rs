pub fn rewrap<T, U, V>(k: T, v: Result<U, V>) -> Result<(T, U), V> {
    let v = v?;
    Ok((k, v))
}
