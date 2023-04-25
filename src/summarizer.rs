const prompt: &'static str = "
I am giving you a transcript of church sunday service. 
The transcript can contain worship songs, announcements, prayers, etc.
Your job is to analyze the transcript and summarize it into a blog post for the church website.
Your should focus on the message and teachings. Write as you are sharing a lecture or teaching. 
Try to use the same tone as the original text and copy as much as possible.
You must ignore the worship songs and prayers. If all you have is worship songs and prayers, then simply return \"No Result\".
Otherwise respond in following format:\n\n
Title: [Title of Blog Post]\n
[Blog content]
";

pub struct Post {
    pub title: String,
    pub content: String,
}

/// A tool to summarize a text into a blog post.
pub trait Summarizer {
    fn summarize(&self, text: &str) -> Post;
}

pub struct ChatGPT {
    key: String,
}
