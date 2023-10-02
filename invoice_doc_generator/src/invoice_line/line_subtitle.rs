use crate::invoice_line::line_subtitle::LineSubtitleError::Empty;
use thiserror::Error;

#[derive(Debug)]
pub struct LineSubtitle(String);
#[derive(Debug, Error)]
pub enum LineSubtitleError {
    #[error("line subtitle cannot be empty")]
    Empty,
    #[error("line subtitle should not be more than {0} char")]
    TooLong(u16),
}
impl LineSubtitle {
    pub fn new(subtitle: String) -> Result<Self, LineSubtitleError> {
        let subtitle= subtitle.trim();
        if subtitle.is_empty() {
            return Err(Empty);
        }
        if subtitle.chars().count() >= 80 {
            return Err(LineSubtitleError::TooLong(80));
        }
        Ok(Self(subtitle.to_string()))
    }
}


#[cfg(test)]
mod line_subtitle_tests{
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;
    use crate::invoice_line::line_subtitle::LineSubtitle;

    #[rstest]
    #[case("",false)]
    #[case("    ",false)]
    #[case("dfafdakjfdfafdafadhdakjfkajlkjweijfojvodidfafdakjfdfafdafadhodafj;ldjd;lajflahvoij;j",false)]
    #[case("kjlj",true)]
    fn test_line_subtitle(#[case] input:String,#[case] valid:bool){
        let line_no = LineSubtitle::new(input);
        if valid {
            assert_that!(line_no).is_ok();
        } else {
            assert_that!(line_no).is_err();
        }
    }
}