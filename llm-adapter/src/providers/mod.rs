pub mod mock;
pub mod openai;
pub mod deepseek;
pub mod doubao;
pub mod zhipu;
pub mod qianwen;

pub use mock::MockAdapter;
#[allow(unused_imports)]
pub use openai::OpenAIAdapter;
#[allow(unused_imports)]
pub use deepseek::DeepSeekAdapter;
#[allow(unused_imports)]
pub use doubao::DoubaoAdapter;
#[allow(unused_imports)]
pub use zhipu::ZhipuAdapter;
#[allow(unused_imports)]
pub use qianwen::QianwenAdapter;
