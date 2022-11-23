
pub struct RecordT<A = ()> {
    pub(crate) slow5_rec: *mut slow5_rec_t,
    _aux: PhantomData<A>,
}

impl<A> RecPtr for RecordT<A> {
    fn ptr(&self) -> RecordPointer {
        RecordPointer { ptr: self.slow5_rec }
    }
}

impl<A> RecordExt for RecordT<A> {}

impl<A> RecordT<A> {

    pub fn get_aux_field<T>(&self, name: &str) -> Result<T, Slow5Error>
    where
        T: AuxField,
    {
        T::aux_get(self, name)
    }
}