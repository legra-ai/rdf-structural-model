//! A single-pass RFC 5646 well-formedness parser that names failing subtags.

use std::iter::{
    Enumerate,
    Peekable,
};
use std::str::Split;

use super::error::{
    LanguageTagError,
    LanguageTagErrorReason as Reason,
    SubtagIndex,
};
use super::grandfathered::IrregularTag;

/// The located parts of a well-formed tag; private-use and irregular
/// grandfathered tags have no language parts.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct LanguageTagParts<'a> {
    pub(super) primary: Option<&'a str>,
    pub(super) script: Option<&'a str>,
    pub(super) region: Option<&'a str>,
    /// The first variant subtag position and the number of variants.
    pub(super) variants: (usize, usize),
}

/// Parses one tag; each method consumes one production of the grammar.
pub(super) struct TagParser<'a> {
    tag: &'a str,
    subtags: Peekable<Enumerate<Split<'a, char>>>,
}

impl<'a> TagParser<'a> {
    /// Check the tag and locate its parts.
    pub(super) fn parse(tag: &'a str) -> Result<LanguageTagParts<'a>, LanguageTagError> {
        Self::check_subtag_shapes(tag)?;
        if IrregularTag::contains(tag) {
            return Ok(LanguageTagParts::default());
        }
        let mut parser = Self {
            tag,
            subtags: tag.split('-').enumerate().peekable(),
        };
        parser.parse_langtag()
    }

    fn check_subtag_shapes(tag: &str) -> Result<(), LanguageTagError> {
        for (index, subtag) in tag.split('-').enumerate() {
            let reason = if subtag.is_empty() {
                Reason::EmptySubtag
            } else if !subtag.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
                Reason::InvalidCharacter
            } else if subtag.len() > 8 {
                Reason::SubtagTooLong
            } else {
                continue;
            };
            return Err(LanguageTagError::new(reason, SubtagIndex::new(index)));
        }
        Ok(())
    }

    fn parse_langtag(&mut self) -> Result<LanguageTagParts<'a>, LanguageTagError> {
        let Some((index, primary)) = self.subtags.next() else {
            return Err(Self::error(Reason::EmptySubtag, 0));
        };
        if primary.eq_ignore_ascii_case("x") {
            self.parse_private_use(index)?;
            return Ok(LanguageTagParts::default());
        }
        if primary.len() < 2 || !Self::is_alpha(primary) {
            return Err(Self::error(Reason::InvalidPrimaryLanguage, index));
        }
        if primary.len() <= 3 {
            for _ in 0..3 {
                self.take_if(|subtag| subtag.len() == 3 && Self::is_alpha(subtag));
            }
        }
        let script = self.take_if(|subtag| subtag.len() == 4 && Self::is_alpha(subtag));
        let region = self.take_if(|subtag| {
            (subtag.len() == 2 && Self::is_alpha(subtag))
                || (subtag.len() == 3 && subtag.bytes().all(|byte| byte.is_ascii_digit()))
        });
        let variants = self.parse_variants()?;
        self.parse_extensions()?;
        if let Some((index, _)) = self.subtags.next() {
            return Err(Self::error(Reason::UnexpectedSubtag, index));
        }
        Ok(LanguageTagParts {
            primary: Some(primary),
            script,
            region,
            variants,
        })
    }

    fn parse_variants(&mut self) -> Result<(usize, usize), LanguageTagError> {
        let start = self.subtags.peek().map_or(0, |(index, _)| *index);
        let mut count = 0;
        while let Some((index, variant)) = self.subtags.next_if(|(_, subtag)| {
            subtag.len() >= 5 || (subtag.len() == 4 && subtag.as_bytes()[0].is_ascii_digit())
        }) {
            if self
                .earlier(start, index)
                .any(|v| v.eq_ignore_ascii_case(variant))
            {
                return Err(Self::error(Reason::DuplicateVariant, index));
            }
            count += 1;
        }
        Ok((start, count))
    }

    fn parse_extensions(&mut self) -> Result<(), LanguageTagError> {
        let start = self.subtags.peek().map_or(0, |(index, _)| *index);
        while let Some((index, singleton)) = self.subtags.next_if(|(_, subtag)| subtag.len() == 1) {
            if singleton.eq_ignore_ascii_case("x") {
                return self.parse_private_use(index);
            }
            if self
                .earlier(start, index)
                .any(|previous| previous.len() == 1 && previous.eq_ignore_ascii_case(singleton))
            {
                return Err(Self::error(Reason::DuplicateExtensionSingleton, index));
            }
            let mut payload = 0_usize;
            while self.take_if(|subtag| subtag.len() >= 2).is_some() {
                payload += 1;
            }
            if payload == 0 {
                return Err(Self::error(Reason::EmptyExtension, index));
            }
        }
        Ok(())
    }

    /// Private-use subtags are 1-8 alphanumerics (RFC 5646 §2.1,
    /// `privateuse`); the length bound is enforced for every subtag up front.
    fn parse_private_use(&mut self, singleton: usize) -> Result<(), LanguageTagError> {
        if self.subtags.next().is_none() {
            return Err(Self::error(Reason::EmptyPrivateUse, singleton));
        }
        // The remaining subtags already passed the 1-8 alphanumeric check.
        self.subtags.by_ref().for_each(drop);
        Ok(())
    }

    fn take_if(&mut self, accept: impl Fn(&str) -> bool) -> Option<&'a str> {
        self.subtags
            .next_if(|(_, subtag)| accept(subtag))
            .map(|(_, subtag)| subtag)
    }

    fn earlier(&self, start: usize, end: usize) -> impl Iterator<Item = &'a str> {
        self.tag.split('-').skip(start).take(end - start)
    }

    fn is_alpha(subtag: &str) -> bool {
        subtag.bytes().all(|byte| byte.is_ascii_alphabetic())
    }

    const fn error(reason: Reason, index: usize) -> LanguageTagError {
        LanguageTagError::new(reason, SubtagIndex::new(index))
    }
}
