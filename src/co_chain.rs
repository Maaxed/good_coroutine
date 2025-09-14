use super::*;
use std::collections::VecDeque;
use std::marker::PhantomData;
use variadics_please::{all_tuples, all_tuples_with_size};


pub fn co_chain<Ctx, Co, Input, C>(coroutine: Co) -> Co::Coroutine
where
	Co: IntoCoChain<Ctx, Input, Coroutine = C>,
	C: Coroutine<Ctx, ()>,
{
	coroutine.into_co_chain()
}

pub struct CoChain<A, B>(A, B);

impl <Ctx, A, B, Input, Mid, Output> Coroutine<Ctx, Input> for CoChain<A, B>
where
	A: Coroutine<Ctx, Input, Output = Mid>,
	B: Coroutine<Ctx, Mid, Output = Output>,
{
	type Output = Output;
	type State = CoChainState<A::State, B, B::State>;

	fn init(self, ctx: &mut Ctx, input: Input) -> CoResult<Self::State, Self::Output>
	{
		let Self(a, b) = self;
		match a.init(ctx, input)
		{
			CoResult::RunNextFrame(a) => CoResult::RunNextFrame(CoChainState::AB(a, b)),
			CoResult::Stop(res) => match b.init(ctx, res)
			{
				CoResult::RunNextFrame(b) => CoResult::RunNextFrame(CoChainState::B(b)),
				CoResult::Stop(res) => CoResult::Stop(res),
			},
		}
	}
}

pub enum CoChainState<AS, B, BS>
{
	AB(AS, B),
	B(BS),
}

impl<Ctx, AS, B, Mid, Output> CoroutineState<Ctx> for CoChainState<AS, B, B::State>
where
	AS: CoroutineState<Ctx, Output = Mid>,
	B: Coroutine<Ctx, Mid, Output = Output>,
{
	type Output = Output;

	fn resume(self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
	{
		let res = match self
		{
			Self::AB(a, b) => match a.resume(ctx)
			{
				CoResult::Stop(res) => b.init(ctx, res),
				CoResult::RunNextFrame(a) => return CoResult::RunNextFrame(Self::AB(a, b)),
			},
			Self::B(b) => b.resume(ctx),
		};

		match res
		{
			CoResult::RunNextFrame(b) => CoResult::RunNextFrame(Self::B(b)),
			CoResult::Stop(res) => CoResult::Stop(res),
		}
	}
}



pub trait IntoCoChain<Ctx, Input>: Sized
{
	type Coroutine: Coroutine<Ctx, Input>;

	fn into_co_chain(self) -> Self::Coroutine;
}


macro_rules! impl_co_chain_tuple
{
	($(($Co:ident, $var:ident)),*) =>
	{
		impl<Ctx, Input, CoFirst, $($Co),*> IntoCoChain<Ctx, Input> for (CoFirst, $($Co,)*)
		where
			CoFirst: Coroutine<Ctx, Input>,
			($($Co,)*): IntoCoChain<Ctx, CoFirst::Output>
		{
			type Coroutine = CoChain<
				CoFirst,
				<($($Co,)*) as IntoCoChain<Ctx, CoFirst::Output>>::Coroutine
			>;

			fn into_co_chain(self) -> Self::Coroutine
			{
				let (
					var_first,
					$( $var, )*
				) = self;

				CoChain(var_first, ($($var,)*).into_co_chain())
			}
		}
	}
}

impl<Ctx, Input, T> IntoCoChain<Ctx, Input> for (T,)
where
	T: Coroutine<Ctx, Input>,
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
		impl<Ctx, Input, Co> IntoCoChain<Ctx, Input> for [Co; $size]
		where
			Co: Coroutine<Ctx, Input, Output = Input>,
		{
			type Coroutine = CoChain<
				Co,
				<[Co; $size-1] as IntoCoChain<Ctx, Input>>::Coroutine
			>;

			fn into_co_chain(self) -> Self::Coroutine
			{
				let [var_first, var_tail @ ..] = self;

				CoChain(var_first, var_tail.into_co_chain())
			}
		}
	}
}

impl<Ctx, Input, Co> IntoCoChain<Ctx, Input> for [Co; 1]
where
	Co: Coroutine<Ctx, Input, Output = Input>,
{
	type Coroutine = Co;

	fn into_co_chain(self) -> Self::Coroutine
	{
		let [co] = self;
		co
	}
}

pub struct CoIdentity;

impl<Ctx, Input> Coroutine<Ctx, Input> for CoIdentity
{
	type Output = Input;
	type State = CoNeverWithOutput<Input>;

	fn init(self, _ctx: &mut Ctx, input: Input) -> CoResult<Self::State, Self::Output>
	{
		co_return(input)
	}
}

impl<Ctx, Input, Co> IntoCoChain<Ctx, Input> for [Co; 0]
where
	Co: Coroutine<Ctx, Input, Output = Input>,
{
	type Coroutine = CoIdentity;

	fn into_co_chain(self) -> Self::Coroutine
	{
		CoIdentity
	}
}

all_tuples_with_size!(impl_co_chain_array, 2, 10, Co, var);



impl<Ctx, Co, Input> IntoCoChain<Ctx, Input> for Vec<Co>
where
	Co: Coroutine<Ctx, Input, Output = Input>,
{
	type Coroutine = CoChainVec<Co>;

	fn into_co_chain(self) -> Self::Coroutine
	{
		CoChainVec(self)
	}
}

pub struct CoChainVec<Co>(Vec<Co>);

impl<Ctx, Co, Input> Coroutine<Ctx, Input> for CoChainVec<Co>
where
	Co: Coroutine<Ctx, Input, Output = Input>,
{
	type Output = Input;
	type State = CoChainVecState<Co, Co::State, Input>;

	fn init(self, ctx: &mut Ctx, mut input: Input) -> CoResult<Self::State, Self::Output>
	{
		let mut vec: VecDeque<_> = self.0.into();
		while let Some(co) = vec.pop_front()
		{
			match co.init(ctx, input)
			{
				CoResult::RunNextFrame(co) =>
				{
					return CoResult::RunNextFrame(CoChainVecState(co, vec, PhantomData))
				},
				CoResult::Stop(res) => input = res,
			}
		}

		CoResult::Stop(input)
	}
}

pub struct CoChainVecState<C, CS, I>(CS, VecDeque<C>, PhantomData<fn(I)>);

impl<Ctx, Co, Input> CoroutineState<Ctx> for CoChainVecState<Co, Co::State, Input>
where
	Co: Coroutine<Ctx, Input, Output = Input>,
{
	type Output = Input;

	fn resume(mut self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
	{
		let mut input = match self.0.resume(ctx)
		{
			CoResult::RunNextFrame(co) =>
			{
				self.0 = co;
				return CoResult::RunNextFrame(self);
			},
			CoResult::Stop(res) => res,
		};
		
		while let Some(co) = self.1.pop_front()
		{
			match co.init(ctx, input)
			{
				CoResult::RunNextFrame(co) =>
				{
					self.0 = co;
					return CoResult::RunNextFrame(self);
				},
				CoResult::Stop(res) => input = res,
			}
		}

		CoResult::Stop(input)
	}
}
