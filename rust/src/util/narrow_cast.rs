pub fn narrow_cast<Out, In: TryInto<Out>>(x: In) -> Out {
    match x.try_into() {
        Ok(x) => x,
        Err(_) => {
            // TODO(port): Include source location information.
            panic!("number not in range");
        }
    }
}
