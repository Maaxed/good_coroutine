use super::*;

pub struct CoParallel<A, B>(A, B);

impl<A, B> CoParallel<A, B>
{
	pub(crate) fn new(a: A, b: B) -> Self
	{
		Self(a, b)
	}
}

impl <Ctx, A, B, AI, BI> Coroutine<Ctx, (AI, BI)> for CoParallel<A, B>
where
	A: Coroutine<Ctx, AI>,
	B: Coroutine<Ctx, BI>,
{
	type Output = (A::Output, B::Output);
	type State = CoParallelState<A::State, B::State, A::Output, B::Output>;

	fn init(self, ctx: &mut Ctx, (ai, bi): (AI, BI)) -> CoResult<Self::State, Self::Output>
	{
		let Self(a, b) = self;

		check_completion(a.init(ctx, ai), b.init(ctx, bi))
	}
}

fn check_completion<Ctx, A, B, AO, BO>(a: CoResult<A, AO>, b: CoResult<B, BO>) -> CoResult<CoParallelState<A, B, AO, BO>, (AO, BO)>
where
	A: CoroutineState<Ctx, Output = AO>,
	B: CoroutineState<Ctx, Output = BO>,
{
	match (a, b)
	{
		(CoResult::Stop(res_a), CoResult::Stop(res_b)) => CoResult::Stop((res_a, res_b)),
		(CoResult::RunNextFrame(a), CoResult::Stop(res_b)) => CoResult::RunNextFrame(CoParallelState::A(a, res_b)),
		(CoResult::Stop(res_a), CoResult::RunNextFrame(b)) => CoResult::RunNextFrame(CoParallelState::B(res_a, b)),
		(CoResult::RunNextFrame(a), CoResult::RunNextFrame(b)) => CoResult::RunNextFrame(CoParallelState::AB(a, b)),
	}
}

pub enum CoParallelState<A, B, AO, BO>
{
	AB(A, B),
	A(A, BO),
	B(AO, B),
}

impl<Ctx, A, B> CoroutineState<Ctx> for CoParallelState<A, B, A::Output, B::Output>
where
	A: CoroutineState<Ctx>,
	B: CoroutineState<Ctx>,
{
	type Output = (A::Output, B::Output);

	fn resume(self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
	{
		let (a, b) = match self
		{
			Self::AB(a, b) => (CoResult::RunNextFrame(a), CoResult::RunNextFrame(b)),
			Self::A(a, res_b) => (CoResult::RunNextFrame(a), CoResult::Stop(res_b)),
			Self::B(res_a, b) => (CoResult::Stop(res_a), CoResult::RunNextFrame(b)),
		};

		let new_a = match a
		{
			CoResult::Stop(res) => CoResult::Stop(res),
			CoResult::RunNextFrame(a) => a.resume(ctx),
		};

		let new_b = match b
		{
			CoResult::Stop(res) => CoResult::Stop(res),
			CoResult::RunNextFrame(b) => b.resume(ctx),
		};

		check_completion(new_a, new_b)
	}
}
