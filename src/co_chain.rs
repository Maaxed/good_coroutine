use super::*;
use variadics_please::{all_tuples, all_tuples_with_size};


pub fn co_chain<Ctx, Co, M>(coroutine: Co) -> IgnoreOutput<Co::Coroutine>
where
	Co: IntoCoChain<Ctx, M>,
{
	IgnoreOutput(co_chain_with_output(coroutine))
}

pub fn co_chain_with_output<Ctx, Co, M>(coroutine: Co) -> Co::Coroutine
where
	Co: IntoCoChain<Ctx, M>,
{
	coroutine.into_co_chain()
}

pub enum CoChain<A, B, F>
{
	A(A, F),
	B(B),
}

impl <Ctx, A, B, F, Output> Coroutine<Ctx> for CoChain<A, B, F>
where
	A: Coroutine<Ctx>,
	B: Coroutine<Ctx, Output = Output>,
	F: FnOnce(A::Output) -> B,
{
	type Output = Output;

	fn resume(self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
	{
		let b = match self
		{
			Self::A(a, f) => match a.resume(ctx)
			{
				CoResult::Stop(res) => f(res),
				CoResult::RunNextFrame(a) => return CoResult::RunNextFrame(Self::A(a, f)),
			},
			Self::B(b) => b,
		};

		match b.resume(ctx)
		{
			CoResult::RunNextFrame(b) => CoResult::RunNextFrame(Self::B(b)),
			CoResult::Stop(res) => CoResult::Stop(res),
		}
	}
}

pub struct IdentityFn<T>(T);

impl <Ctx, A, B, Output> Coroutine<Ctx> for CoChain<A, B, IdentityFn<B>>
where
	A: Coroutine<Ctx>,
	B: Coroutine<Ctx, Output = Output>,
{
	type Output = Output;

	fn resume(self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
	{
		let b = match self
		{
			Self::A(a, f) => match a.resume(ctx)
			{
				CoResult::Stop(_res) => f.0,
				CoResult::RunNextFrame(a) => return CoResult::RunNextFrame(Self::A(a, f)),
			},
			Self::B(b) => b,
		};

		match b.resume(ctx)
		{
			CoResult::RunNextFrame(b) => CoResult::RunNextFrame(Self::B(b)),
			CoResult::Stop(res) => CoResult::Stop(res),
		}
	}
}



pub trait IntoCoChain<Ctx, M>: Sized
{
	type Coroutine: Coroutine<Ctx>;

	fn into_co_chain(self) -> Self::Coroutine;
}

pub struct FnMarker;
pub struct CoMarker;

macro_rules! impl_co_chain_tuple
{
	($(($Co:ident, $var:ident)),*) =>
	{
		impl<Ctx, Chain, CoLast, F, I, M, $($Co,)*> IntoCoChain<Ctx, (M, FnMarker)> for ($($Co,)* F,)
		where
			F: FnOnce(I) -> CoLast,
			CoLast: Coroutine<Ctx>,
			Chain: Coroutine<Ctx, Output = I>,
			($($Co,)*): IntoCoChain<Ctx, M, Coroutine = Chain>
		{
			type Coroutine = CoChain<
				Chain,
				CoLast,
				F
			>;

			fn into_co_chain(self) -> Self::Coroutine
			{
				let (
					$( $var, )*
					var_last,
				) = self;

				CoChain::A(($($var,)*).into_co_chain(), var_last)
			}
		}

		impl<Ctx, Chain, CoLast, M, $($Co,)*> IntoCoChain<Ctx, (M, CoMarker)> for ($($Co,)* CoLast,)
		where
			CoLast: Coroutine<Ctx>,
			Chain: Coroutine<Ctx>,
			($($Co,)*): IntoCoChain<Ctx, M, Coroutine = Chain>
		{
			type Coroutine = CoChain<
				Chain,
				CoLast,
				IdentityFn<CoLast>
			>;

			fn into_co_chain(self) -> Self::Coroutine
			{
				let (
					$( $var, )*
					var_last,
				) = self;

				CoChain::A(($($var,)*).into_co_chain(), IdentityFn(var_last))
			}
		}
	}
}

impl<Ctx, T> IntoCoChain<Ctx, ()> for (T,)
where
	T: Coroutine<Ctx>,
{
	type Coroutine = T;

	fn into_co_chain(self) -> Self::Coroutine
	{
		self.0
	}
}

all_tuples!(impl_co_chain_tuple, 1, 9, Co, var);


macro_rules! impl_co_chain_array
{
	($size:tt, $(($Co:ident, $var:ident)),*) =>
	{
		impl<Ctx, Co> IntoCoChain<Ctx, [(); $size]> for [Co; $size]
		where
			Co: Coroutine<Ctx>,
		{
			type Coroutine = CoChain<
				<[Co; $size-1] as IntoCoChain<Ctx, [(); $size-1]>>::Coroutine,
				Co,
				IdentityFn<Co>,
			>;

			fn into_co_chain(self) -> Self::Coroutine
			{
				let [var_head @ .., var_last] = self;

				CoChain::A(var_head.into_co_chain(), IdentityFn(var_last))
			}
		}
	}
}

impl<Ctx, Co> IntoCoChain<Ctx, [(); 1]> for [Co; 1]
where
	Co: Coroutine<Ctx>,
{
	type Coroutine = Co;

	fn into_co_chain(self) -> Self::Coroutine
	{
		let [co] = self;
		co
	}
}

all_tuples_with_size!(impl_co_chain_array, 2, 10, Co, var);



impl<Ctx, Co> IntoCoChain<Ctx, Vec<()>> for Vec<Co>
where
	Co: Coroutine<Ctx>,
{
	type Coroutine = CoChainIter<std::vec::IntoIter<Co>>;

	fn into_co_chain(self) -> Self::Coroutine
	{
		CoChainIter::new(self.into_iter())
	}
}

pub struct CoChainIter<I: Iterator>
{
	current: Option<I::Item>,
	iter: I,
}

impl<I: Iterator> CoChainIter<I>
{
	pub fn new(iter: I) -> Self
	{
		Self
		{
			current: None,
			iter,
		}
	}
}

impl<Ctx, Co, Iter> Coroutine<Ctx> for CoChainIter<Iter>
where
	Co: Coroutine<Ctx>,
	Iter: Iterator<Item = Co>,
{
	type Output = Option<Co::Output>;

	fn resume(mut self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
	{
		let mut co = match self.current.take()
		{
			Some(co) => co,
			None => match self.iter.next()
			{
				// self.current May be none on the first call to resume
				// Return none if the iter is empty
				None => return CoResult::Stop(None),
				Some(co) => co,
			}
		};

		loop
		{
			match co.resume(ctx)
			{
				CoResult::RunNextFrame(co) =>
				{
					self.current = Some(co);
					return CoResult::RunNextFrame(self);
				},
				CoResult::Stop(res) =>
				{
					match self.iter.next()
					{
						None => return CoResult::Stop(Some(res)),
						Some(next_co) => co = next_co,
					}
				},
			}
		}
	}
}
