use crate::user::*;

#[derive(Default)]
pub struct Locale {
    pub begin: &'static str,
    pub setup: &'static str,
    pub close: &'static str,
    pub accept: &'static str,
    pub cancel: &'static str,
    pub locale_kind: &'static str,
}

impl Locale {
    pub fn new(locale_kind: LocaleKind) -> Self {
        match locale_kind {
            LocaleKind::English => Self {
                begin: "begin",
                setup: "setup",
                close: "close",
                accept: "accept",
                cancel: "cancel",
                locale_kind: "language",
            },
            LocaleKind::Spanish => Self {
                begin: "comenzar",
                setup: "configuración",
                close: "terminar",
                accept: "aceptar",
                cancel: "cancelar",
                locale_kind: "lenguaje",
            },
        }
    }
}
