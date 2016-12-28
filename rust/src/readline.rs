use rustyline;
use term;
use std::env;
use std::fs::File;
use std::io;
use std::io::Write;


#[cfg(not(feature = "readline"))]
pub type Reader = DefaultReader;

#[cfg(feature = "readline")]
pub type Reader = LineReader;


pub struct DefaultReader {
    prompt: String,
}

impl DefaultReader {
    pub fn new(prompt: String) -> DefaultReader {
        DefaultReader { prompt: prompt }
    }

    pub fn read(&mut self) -> Option<String> {
        let mut t = term::stdout().unwrap();

        t.fg(term::color::YELLOW).unwrap();
        t.attr(term::Attr::Bold).unwrap();
        write!(t, "{}", self.prompt).unwrap();

        t.reset().unwrap();

        io::stdout()
            .flush()
            .expect("could not flush line");

        let mut input = String::new();
        let read = io::stdin().read_line(&mut input);

        match read {
            // catches CTRL-D
            Ok(n) => {
                if n <= 0 {
                    None
                } else {
                    Some(input.trim().to_string())
                }
            }
            Err(_) => None,
        }
    }
}

type LineEditor = rustyline::Editor<()>;

pub struct LineReader {
    prompt: String,
    editor: LineEditor,
    history_path: String,
}

const HISTORY_FILENAME: &'static str = ".mal-history.txt";

impl LineReader {
    pub fn new(prompt: String) -> LineReader {
        let mut editor = LineEditor::new();

        let path = LineReader::init_history(&mut editor, HISTORY_FILENAME).unwrap();

        LineReader {
            prompt: prompt,
            editor: editor,
            history_path: path,
        }
    }

    fn init_history(editor: &mut LineEditor,
                    filename: &str)
                    -> Result<String, rustyline::error::ReadlineError> {
        let mut path = env::home_dir().unwrap();
        path.push(filename);

        if !path.exists() {
            File::create(&path).unwrap();
        }

        editor.save_history(&path).unwrap();
        editor.load_history(&path)
            .and_then(|_| {
                Ok(path.as_path()
                    .to_str()
                    .unwrap()
                    .to_string())
            })
    }

    pub fn read(&mut self) -> Option<String> {
        let readline = self.editor.readline(&self.prompt);
        match readline {
            Ok(line) => {
                self.editor.add_history_entry(&line);
                Some(line)
            }
            Err(..) => None,
        }
    }
}

impl Drop for LineReader {
    fn drop(&mut self) {
        self.editor.save_history(&self.history_path).unwrap();
    }
}
