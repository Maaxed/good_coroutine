use super::*;

pub struct CoChain<A, B>(A, B);

impl<A, B> CoChain<A, B>
{
	pub(crate) fn new(a: A, b: B) -> Self
	{
		Self(a, b)
	}
}

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
