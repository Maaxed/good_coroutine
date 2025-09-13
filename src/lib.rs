mod impls;
mod co_function;
mod co_chain;
mod co_never;
mod co_box;
mod co_loop;
mod co_vec;
mod co_parallel;

pub use impls::*;
pub use co_function::*;
pub use co_chain::*;
pub use co_never::*;
pub use co_box::*;
pub use co_loop::*;
pub use co_vec::*;
pub use co_parallel::*;

#[must_use]
pub trait Coroutine<Ctx>: Sized
{
	fn resume(self, ctx: &mut Ctx) -> CoResult<Self>;
}


#[must_use]
pub enum CoResult<Co = CoNever>
{
	Stop,
	RunNextFrame(Co),
}

impl<Co> From<CoResult<Co>> for Option<Co>
{
	fn from(value: CoResult<Co>) -> Self
	{
		match value
		{
			CoResult::Stop => None,
			CoResult::RunNextFrame(co) => Some(co),
		}
	}
}


pub trait IntoCoroutine<Ctx, Marker>: Sized
{
	type Coroutine: Coroutine<Ctx>;

	fn into_coroutine(self) -> Self::Coroutine;
}


pub fn co_return<Co>() -> CoResult<Co>
{
	CoResult::Stop
}

pub fn co_yield<Ctx, Co, M>(coroutine: Co, ctx: &mut Ctx) -> CoResult<Co::Coroutine>
where
	Co: IntoCoroutine<Ctx, M>,
{
	coroutine.into_coroutine().resume(ctx)
}

pub struct CoNextFrame(bool);

impl<Ctx> Coroutine<Ctx> for CoNextFrame
{
	fn resume(self, _ctx: &mut Ctx) -> CoResult<Self>
	{
		if self.0
		{
			co_return()
		}
		else
		{
			CoResult::RunNextFrame(Self(true))
		}
	}
}

pub fn co_next_frame() -> CoNextFrame
{
	CoNextFrame(false)
}

#[cfg(test)]
mod tests
{
	use crate::*;

	#[test]
	fn basic_coroutine()
	{
		fn coroutine_function() -> impl Coroutine<Vec<u32>>
		{
			co_fn(|ctx|
			{
				let mut vec = co_vec();

				for i in 0..4
				{
					vec.push(co_fn(move |ctx: &mut Vec<u32>| -> CoResult
					{
						ctx.push(i);

						co_return()
					}));
				}

				co_yield(vec, ctx)
			})
		}

		let coroutine = coroutine_function();

		let mut ctx = Vec::new();
		let res = coroutine.resume(&mut ctx);
		assert!(matches!(res, CoResult::Stop));

		assert_eq!(ctx, vec![0, 1, 2, 3]);
	}

	#[test]
	fn coroutine_next_frame()
	{
		fn coroutine_function2() -> impl Coroutine<Vec<u32>>
		{
			co_fn(|ctx|
			{
				let mut vec = co_vec();

				for i in 0..2
				{
					vec.push(co_fn(move |ctx: &mut Vec<u32>|
					{
						ctx.push(i);

						co_yield(co_next_frame(), ctx)
					}));
				}

				co_yield(vec, ctx)
			})
		}

		let coroutine = coroutine_function2();
		let mut ctx = Vec::new();

		let res = coroutine.resume(&mut ctx);
		let CoResult::RunNextFrame(coroutine) = res
		else
		{
			panic!("assertion failed: expected CoResult::RunNextFrame");
		};
		assert_eq!(ctx, vec![0]);

		let res = coroutine.resume(&mut ctx);
		let CoResult::RunNextFrame(coroutine) = res
		else
		{
			panic!("assertion failed: expected CoResult::RunNextFrame");
		};
		assert_eq!(ctx, vec![0, 1]);

		let res = coroutine.resume(&mut ctx);
		assert!(matches!(res, CoResult::Stop));
		assert_eq!(ctx, vec![0, 1]);
	}
}
