pub mod deepseek;
pub mod doubao;
pub mod mock;
pub mod openai;
pub mod qianwen;
pub mod zhipu;

#[allow(unused_imports)]
pub use deepseek::DeepSeekAdapter;
#[allow(unused_imports)]
pub use doubao::DoubaoAdapter;
pub use mock::MockAdapter;
#[allow(unused_imports)]
pub use openai::OpenAIAdapter;
#[allow(unused_imports)]
pub use qianwen::QianwenAdapter;
#[allow(unused_imports)]
pub use zhipu::ZhipuAdapter;
