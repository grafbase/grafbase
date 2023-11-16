use std::num::NonZeroUsize;

#[derive(Debug, clap::Args)]
pub struct BuildCommand {
    /// Number of resolver builds running in parallel
    #[arg(long)]
    pub parallelism: Option<u16>,
}

impl BuildCommand {
    pub fn parallelism(&self) -> NonZeroUsize {
        let parallelism = self.parallelism.unwrap_or(0);
        if parallelism == 0 {
            std::thread::available_parallelism().unwrap_or(NonZeroUsize::new(1).expect("strictly positive"))
        } else {
            NonZeroUsize::new(parallelism as usize).expect("strictly positive")
        }
    }
}
