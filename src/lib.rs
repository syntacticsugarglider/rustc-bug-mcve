use std::{convert::Infallible, marker::PhantomData};

pub trait Sink<T> {
    type Error;
}

pub trait Stream {
    type Item;
}

pub struct ImplSink<T, E> {
    _marker: PhantomData<(T, E)>,
}

impl<T, E> Sink<T> for ImplSink<T, E> {
    type Error = E;
}

pub struct ImplStream<T> {
    _marker: PhantomData<T>,
}

impl<T> Stream for ImplStream<T> {
    type Item = T;
}

pub trait TryStream: Stream {
    type Ok;

    type Error;
}

impl<S, T, E> TryStream for S
where
    S: Stream<Item = Result<T, E>>,
{
    type Ok = T;
    type Error = E;
}

pub trait Coalesce<C: ?Sized>: Sized {
    type Future: Future<C, Ok = Self>;
}
pub struct MapErr<Fut, F> {
    _marker: PhantomData<(Fut, F)>,
}

impl<C: ?Sized, E, Fut: Unpin + Future<C>, F: Unpin + FnMut(Fut::Error) -> E> Future<C>
    for MapErr<Fut, F>
{
    type Ok = Fut::Ok;
    type Error = E;
}

pub trait Future<C: ?Sized> {
    type Ok;
    type Error;
}
pub trait Unravel<C: ?Sized> {
    type Finalize: Future<C, Ok = (), Error = <Self::Target as Future<C>>::Error>;
    type Target: Future<C, Ok = Self::Finalize>;
}

pub trait Dispatch<P> {
    type Handle;
}

pub trait Fork<P>: Dispatch<P> {
    type Finalize: Future<Self, Ok = (), Error = <Self::Target as Future<Self>>::Error>;
    type Target: Future<Self, Ok = Self::Finalize>;
    type Future: Future<Self, Ok = (Self::Target, Self::Handle)>;
}

pub trait Write<T> {
    type Error;
}

pub struct FlatUnravelError<T, U, V> {
    _marker: PhantomData<(T, U, V)>,
}

pub struct FlatUnravelState<T, C: ?Sized + Write<<C as Dispatch<T>>::Handle> + Fork<T> + Unpin> {
    _marker: PhantomData<(T, C)>,
}

pub struct FlatUnravel<T, C: ?Sized + Write<<C as Dispatch<T>>::Handle> + Fork<T> + Unpin> {
    _marker: PhantomData<(T, C)>,
}
impl<T, C: ?Sized + Write<<C as Dispatch<T>>::Handle> + Fork<T> + Unpin> Future<C>
    for FlatUnravel<T, C>
where
    C::Future: Unpin,
    C::Target: Unpin,
    C::Handle: Unpin,
{
    type Ok = MapErr<
        C::Finalize,
        fn(
            <C::Finalize as Future<C>>::Error,
        ) -> FlatUnravelError<
            C::Error,
            <C::Future as Future<C>>::Error,
            <C::Target as Future<C>>::Error,
        >,
    >;
    type Error = FlatUnravelError<
        C::Error,
        <C::Future as Future<C>>::Error,
        <C::Target as Future<C>>::Error,
    >;
}

impl<T, C: ?Sized + Write<<C as Dispatch<T>>::Handle> + Fork<T> + Unpin> Unravel<C> for (T,)
where
    C::Future: Unpin,
    C::Target: Unpin,
    C::Finalize: Unpin,
    C::Handle: Unpin,
{
    type Finalize = MapErr<
        C::Finalize,
        fn(
            <C::Finalize as Future<C>>::Error,
        ) -> FlatUnravelError<
            C::Error,
            <C::Future as Future<C>>::Error,
            <C::Target as Future<C>>::Error,
        >,
    >;
    type Target = FlatUnravel<T, C>;
}

type _5<T> = (((((T,),),),),);
type _15<T> = _5<_5<_5<T>>>;
type Test = _15<_15<_15<()>>>;

pub struct Transport<StreamError, SinkError, P> {
    _marker: PhantomData<(StreamError, SinkError, P)>,
}

pub struct UnravelStruct<
    T: TryStream<Ok = Vec<u8>>,
    U: Sink<Vec<u8>>,
    P: Unravel<Transport<T::Error, U::Error, P>>,
> where
    P::Target: Unpin,
    P::Finalize: Unpin,
{
    _marker: PhantomData<(T, U, P)>,
}

pub struct Tester
where
    UnravelStruct<ImplStream<Result<Vec<u8>, Infallible>>, ImplSink<Vec<u8>, Infallible>, Test>:
        All;

impl<I, T, U, P> Write<I> for Transport<T, U, P> {
    type Error = U;
}

impl<T, U, P: Unravel<Self>, M> Dispatch<P> for Transport<T, U, M> {
    type Handle = ();
}

impl<T, U, P: Unravel<Self>, M> Fork<P> for Transport<T, U, M>
where
    <P as Unravel<Self>>::Target: Unpin,
{
    type Finalize = <P as Unravel<Self>>::Finalize;
    type Target = <P as Unravel<Self>>::Target;
    type Future = Ready<(Self::Target, ())>;
}

pub trait All {}

impl<T> All for T {}

pub struct Ready<T, E = Infallible> {
    _marker: PhantomData<(T, E)>,
}

impl<T: Unpin, E> Unpin for Ready<T, E> {}

impl<C: ?Sized, T: Unpin, E> Future<C> for Ready<T, E> {
    type Ok = T;
    type Error = E;
}

impl<C: ?Sized> Unravel<C> for () {
    type Finalize = Ready<()>;
    type Target = Ready<Ready<()>>;
}
