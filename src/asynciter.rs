pub trait FirstSomeThrowing {
    type Input;
    async fn first_some<F, Error, Output>(self, f: F) -> Result<Option<Output>, Error>
    where
        Error: std::error::Error,
        F: AsyncFnMut(Self::Input) -> Result<Option<Output>, Error>;
}

impl<I> FirstSomeThrowing for I
where
    I: IntoIterator,
{
    type Input = I::Item;

    async fn first_some<F, Error, Output>(self, mut f: F) -> Result<Option<Output>, Error>
    where
        Error: std::error::Error,
        F: AsyncFnMut(Self::Input) -> Result<Option<Output>, Error>,
    {
        for item in self {
            if let Some(item) = f(item).await? {
                return Ok(Some(item));
            }
        }
        return Ok(None);
    }
}
