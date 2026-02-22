use super::*;
use variadics_please::all_tuples_enumerated;


pub fn co_concurrent<Ctx, Co>(coroutine: Co) -> IgnoreOutput<Co::Coroutine>
where
	Co: IntoCoConcurrent<Ctx>,
{
	IgnoreOutput(co_concurrent_with_output(coroutine))
}

pub fn co_concurrent_with_output<Ctx, Co>(coroutine: Co) -> Co::Coroutine
where
	Co: IntoCoConcurrent<Ctx>,
{
	coroutine.into_co_concurrent()
}

pub struct CoConcurrentWithOutput<T>(T);

pub trait IntoCoConcurrent<Ctx>: Sized
{
	type Coroutine: Coroutine<Ctx>;

	fn into_co_concurrent(self) -> Self::Coroutine;
}

mod array
{
	use super::*;

	impl<Ctx, C, const N: usize> IntoCoConcurrent<Ctx> for [C; N]
	where
		C: Coroutine<Ctx>,
	{
		type Coroutine = CoConcurrentWithOutput<[CoResult<C, C::Output>; N]>;

		fn into_co_concurrent(self) -> Self::Coroutine
		{
			CoConcurrentWithOutput(
				self.map(|co| CoResult::RunNextFrame(co))
			)
		}
	}
	
	impl<Ctx, C, const N: usize> Coroutine<Ctx> for CoConcurrentWithOutput<[CoResult<C, C::Output>; N]>
	where
		C: Coroutine<Ctx>
	{
		type Output = [C::Output; N];
	
		fn resume(self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
		{
			let res = Self(
				self.0.map(|res|
					match res
					{
						CoResult::Stop(res) => CoResult::Stop(res),
						CoResult::RunNextFrame(co) => co.resume(ctx),
					}
				)
			);
			
			res.check_completion()
		}
	}
	
	impl<C, Output, const N: usize> CoConcurrentWithOutput<[CoResult<C, Output>; N]>
	{
		fn check_completion<Ctx>(self) -> CoResult<Self, [Output; N]>
		where
			C: Coroutine<Ctx, Output = Output>
		{
			if self.0.iter().all(|res| matches!(res, CoResult::Stop(_)))
			{
				CoResult::Stop(self.0.map(|res|
					match res
					{
						CoResult::Stop(res) => res,
						_ => unreachable!(),
					}
				))
			}
			else
			{
				CoResult::RunNextFrame(self)
			}
		}
	}
}

mod vec
{
	use super::*;

	impl<Ctx, C> IntoCoConcurrent<Ctx> for Vec<C>
	where
		C: Coroutine<Ctx>,
	{
		type Coroutine = CoConcurrentWithOutput<Vec<CoResult<C, C::Output>>>;

		fn into_co_concurrent(self) -> Self::Coroutine
		{
			CoConcurrentWithOutput(
				self.into_iter().map(|co| CoResult::RunNextFrame(co)).collect()
			)
		}
	}

	impl<Ctx, C> Coroutine<Ctx> for CoConcurrentWithOutput<Vec<CoResult<C, C::Output>>>
	where
		C: Coroutine<Ctx>
	{
		type Output = Vec<C::Output>;

		fn resume(self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
		{
			let state = Self(
				self.0.into_iter().map(|res|
					match res
					{
						CoResult::Stop(res) => CoResult::Stop(res),
						CoResult::RunNextFrame(co) => co.resume(ctx),
					}
				).collect()
			);
			
			state.check_completion()
		}
	}

	impl<C, Output> CoConcurrentWithOutput<Vec<CoResult<C, Output>>>
	{
		fn check_completion<Ctx>(self) -> CoResult<Self, Vec<Output>>
		where
			C: Coroutine<Ctx, Output = Output>
		{
			if self.0.iter().all(|res| matches!(res, CoResult::Stop(_)))
			{
				CoResult::Stop(self.0.into_iter().map(|res|
					match res
					{
						CoResult::Stop(res) => res,
						_ => unreachable!(),
					}
				).collect())
			}
			else
			{
				CoResult::RunNextFrame(self)
			}
		}
	}
}

macro_rules! impl_co_concurrent_tuple
{
	($(($i:tt, $C:ident, $Input:ident, $Output:ident, $res:ident)),*) =>
	{
		impl<Ctx, $($C,)*> IntoCoConcurrent<Ctx> for ($($C,)*)
		where
			$( $C: Coroutine<Ctx>, )*
		{
			type Coroutine = CoConcurrentWithOutput<(
				$( CoResult<$C, $C::Output>, )*
			)>;

			fn into_co_concurrent(self) -> Self::Coroutine
			{
				CoConcurrentWithOutput((
					$( CoResult::RunNextFrame(self.$i), )*
				))
			}
		}

		impl<Ctx, $($C),*> Coroutine<Ctx> for CoConcurrentWithOutput<($( CoResult<$C, $C::Output>,)*)>
		where
			$( $C: Coroutine<Ctx>, )*
		{
			type Output = (
				$( $C::Output, )*
			);

			fn resume(self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
			{
				let state = Self((
					$(
						match self.0.$i
						{
							CoResult::Stop(res) => CoResult::Stop(res),
							CoResult::RunNextFrame(co) => co.resume(ctx),
						},
					)*
				));
				
				state.check_completion()
			}
		}

		impl<$($C, $Output),*> CoConcurrentWithOutput<($( CoResult<$C, $Output>,)*)>
		{
			fn check_completion<Ctx>(self) -> CoResult<Self, ($( $C::Output, )*)>
			where
				$( $C: Coroutine<Ctx, Output = $Output>, )*
			{
				if let Self((
					$( CoResult::Stop($res), )*
				)) = self
				{
					CoResult::Stop((
						$( $res, )*
					))
				}
				else
				{
					CoResult::RunNextFrame(self)
				}
			}
		}
	}
}

all_tuples_enumerated!(impl_co_concurrent_tuple, 1, 10, C, Input, Output, res);
