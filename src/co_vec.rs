use super::*;
use std::collections::VecDeque;

pub fn co_vec<Co>(vec: Vec<Co>) -> CoVec<Co>
{
	CoVec(vec)
}

pub struct CoVec<Co>(Vec<Co>);

/*impl<Co> CoVec<Co>
{
	pub fn push<Ctx, M>(&mut self, co: Co)
	{
		self.0.push_back(co.into_coroutine());
	}
}*/

impl<Ctx, Co> Coroutine<Ctx, ()> for CoVec<Co>
where
	Co: Coroutine<Ctx, (), Output = ()>,
{
	type Output = ();
	type State = CoVecState<Co, Co::State>;

	fn init(self, ctx: &mut Ctx, _input: ()) -> CoResult<Self::State, Self::Output>
	{
		let mut vec: VecDeque<_> = self.0.into();
		while let Some(co) = vec.pop_front()
		{
			if let CoResult::RunNextFrame(co) = co.init(ctx, ())
			{
				return CoResult::RunNextFrame(CoVecState(co, vec))
			}
		}

		CoResult::Stop(())
	}
}

pub struct CoVecState<C, CS>(CS, VecDeque<C>);

impl<Ctx, Co> CoroutineState<Ctx> for CoVecState<Co, Co::State>
where
	Co: Coroutine<Ctx, (), Output = ()>,
{
	type Output = ();

	fn resume(mut self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
	{
		if let CoResult::RunNextFrame(co) = self.0.resume(ctx)
		{
			self.0 = co;
			return CoResult::RunNextFrame(self)
		}
		
		while let Some(co) = self.1.pop_front()
		{
			if let CoResult::RunNextFrame(co) = co.init(ctx, ())
			{
				self.0 = co;
				return CoResult::RunNextFrame(self)
			}
		}

		CoResult::Stop(())
	}
}
