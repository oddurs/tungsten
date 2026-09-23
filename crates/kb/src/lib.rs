//! The knowledge base: constants, elements, the solar system and everyday items.
//!
//! Compiled from data/*.toml by build.rs, which validates every value against
//! the unit table, so reading one here cannot fail. See docs/concept.md §4.

use tungsten_units::{Number, Rational, UnitExpr, parse_def};

/// What kind of thing an entity is. Declaration order is the preference when
/// one name means several things: "mercury" is a planet before an element.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    Star,
    Planet,
    DwarfPlanet,
    Moon,
    Element,
    Item,
    Constant,
}

impl Kind {
    pub const ALL: [Self; 7] = [
        Self::Star,
        Self::Planet,
        Self::DwarfPlanet,
        Self::Moon,
        Self::Element,
        Self::Item,
        Self::Constant,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Constant => "constant",
            Self::Element => "element",
            Self::Star => "star",
            Self::Planet => "planet",
            Self::DwarfPlanet => "dwarf planet",
            Self::Moon => "moon",
            Self::Item => "item",
        }
    }

    /// `element`, `dwarf-planet`, `dwarf planet`.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.to_lowercase().replace(['-', '_'], " ");
        Self::ALL.into_iter().find(|k| k.name() == s)
    }
}

#[derive(Debug)]
pub struct PropDef {
    pub names: &'static [&'static str],
    pub show: Option<&'static str>,
}

#[derive(Debug)]
pub struct EntityDef {
    pub kind: Kind,
    pub display: &'static str,
    pub plural: &'static str,
    pub names: &'static [&'static str],
    pub symbols: &'static [&'static str],
    pub default: Option<u16>,
    pub scale: &'static [u16],
    pub domain: Option<&'static str>,
    pub source: &'static str,
    /// (property, value, uncertainty, source override)
    pub props: &'static [(
        u16,
        &'static str,
        Option<&'static str>,
        Option<&'static str>,
    )],
    /// (name, text, source override)
    pub facts: &'static [(&'static str, &'static str, Option<&'static str>)],
}

include!(concat!(env!("OUT_DIR"), "/kb.rs"));

/// A property: `mass`, `melting point`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Prop(u16);

impl Prop {
    fn def(self) -> &'static PropDef {
        &PROPS[self.0 as usize]
    }

    pub fn name(self) -> &'static str {
        self.def().names[0]
    }

    /// The unit values of this property are shown in, if not their own.
    pub fn show(self) -> Option<UnitExpr> {
        self.def().show.and_then(UnitExpr::parse)
    }
}

/// An entity: Earth, gold, the Newtonian constant of gravitation, a banana.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity(u16);

/// One value of one entity.
#[derive(Clone, Debug)]
pub struct Value {
    pub prop: Prop,
    /// The number, in `unit` (not SI).
    pub num: Number,
    pub unit: UnitExpr,
    /// Standard uncertainty as published, or `exact`.
    pub uncertainty: Option<&'static str>,
    pub source: &'static str,
    /// Significant digits in the value as written: 6.67430e-11 has six. `None`
    /// for integers with trailing zeros, whose precision the text cannot say.
    pub digits: Option<u32>,
}

/// Significant digits of the number at the start of `text`: `6.67430e-11 …` → 6,
/// `0.0012 g` → 2, `8611 m` → 4, `330000 lb` → `None` (trailing zeros of an
/// integer are ambiguous).
pub fn significant_digits(text: &str) -> Option<u32> {
    let num = text.split_whitespace().next()?;
    let mantissa = num
        .trim_start_matches(['-', '+'])
        .split(['e', 'E'])
        .next()?;
    if mantissa.is_empty() || !mantissa.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let digits = digits.trim_start_matches('0');
    if !mantissa.contains('.') && digits.ends_with('0') {
        return None;
    }
    u32::try_from(digits.len()).ok().filter(|n| *n > 0)
}

/// A text fact: `state: solid`.
#[derive(Clone, Copy, Debug)]
pub struct Fact {
    pub name: &'static str,
    pub text: &'static str,
    pub source: &'static str,
}

impl Entity {
    fn def(self) -> &'static EntityDef {
        &ENTITIES[self.0 as usize]
    }

    pub fn kind(self) -> Kind {
        self.def().kind
    }

    /// How to write it: `Earth`, `gold`, `the Moon`.
    pub fn display(self) -> &'static str {
        self.def().display
    }

    pub fn plural(self) -> &'static str {
        self.def().plural
    }

    pub fn symbol(self) -> Option<&'static str> {
        self.def().symbols.first().copied()
    }

    pub fn source(self) -> &'static str {
        self.def().source
    }

    pub fn domain(self) -> Option<&'static str> {
        self.def().domain
    }

    /// The property a bare mention stands for: `3 coffees` is 3 × caffeine.
    pub fn default(self) -> Option<Prop> {
        self.def().default.map(Prop)
    }

    /// Properties this entity lends to the "for scale" pod.
    pub fn scale(self) -> impl Iterator<Item = Prop> {
        self.def().scale.iter().map(|p| Prop(*p))
    }

    pub fn has(self, p: Prop) -> bool {
        self.def().props.iter().any(|(q, ..)| *q == p.0)
    }

    pub fn value(self, p: Prop) -> Option<Value> {
        let &(_, text, uncertainty, source) = self.def().props.iter().find(|(q, ..)| *q == p.0)?;
        let parsed = parse_def(text).ok()?;
        Some(Value {
            prop: p,
            num: parsed.coef.with_decimal(true),
            unit: UnitExpr::from_terms(parsed.terms),
            uncertainty,
            source: source.unwrap_or(self.def().source),
            digits: significant_digits(text),
        })
    }

    pub fn values(self) -> impl Iterator<Item = Value> {
        self.def()
            .props
            .iter()
            .filter_map(move |(p, ..)| self.value(Prop(*p)))
    }

    pub fn facts(self) -> impl Iterator<Item = Fact> {
        let src = self.def().source;
        self.def().facts.iter().map(move |&(name, text, s)| Fact {
            name,
            text,
            source: s.unwrap_or(src),
        })
    }

    pub fn fact(self, name: &str) -> Option<Fact> {
        self.facts().find(|f| f.name == name)
    }

    /// Resolves a property word for this entity: its own name, or a looser
    /// `also` name ("distance") when exactly one of its properties claims it.
    pub fn resolve_prop(self, candidates: &[Prop]) -> Option<Prop> {
        let mine: Vec<Prop> = candidates
            .iter()
            .copied()
            .filter(|p| self.has(*p))
            .collect();
        match mine.as_slice() {
            [one] => Some(*one),
            _ => None,
        }
    }
}

/// A match for a word: the entity, and whether the word is shadowed by a unit
/// or keyword (then it only counts when asked for with `--as`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    pub entity: Entity,
    pub shadowed: bool,
}

fn range(table: &'static [(&'static str, u16, bool)], key: &str) -> impl Iterator<Item = Hit> {
    let start = table.partition_point(|(k, ..)| *k < key);
    table[start..]
        .iter()
        .take_while(move |(k, ..)| *k == key)
        .map(|&(_, e, shadowed)| Hit {
            entity: Entity(e),
            shadowed,
        })
}

/// Every entity a word could mean: symbols match exactly, names ignore case.
pub fn lookup(word: &str) -> Vec<Hit> {
    let mut hits: Vec<Hit> = range(SYMBOLS, word).collect();
    for h in range(NAMES, &word.to_lowercase()) {
        match hits.iter_mut().find(|x| x.entity == h.entity) {
            // A name that is not shadowed beats a symbol that is.
            Some(x) => x.shadowed &= h.shadowed,
            None => hits.push(h),
        }
    }
    hits
}

/// Properties a word could name. `distance` names several; an entity picks.
pub fn lookup_prop(word: &str) -> Vec<Prop> {
    let key = word.to_lowercase();
    let start = PROP_NAMES.partition_point(|(k, ..)| *k < key.as_str());
    PROP_NAMES[start..]
        .iter()
        .take_while(|(k, ..)| *k == key)
        .map(|&(_, p, _)| Prop(p))
        .collect()
}

/// The longest entity name, in words.
pub fn max_name_words() -> usize {
    MAX_NAME_WORDS
}

pub fn entities() -> impl Iterator<Item = Entity> {
    (0..ENTITIES.len()).map(|i| Entity(i as u16))
}

/// Every name and symbol, for did-you-mean suggestions.
pub fn all_names() -> impl Iterator<Item = &'static str> {
    NAMES
        .iter()
        .chain(SYMBOLS)
        .map(|(n, ..)| *n)
        .chain(PROP_NAMES.iter().map(|(n, ..)| *n))
}

/// Exactness helper for callers building values: a plain count.
pub fn count(n: i128) -> Number {
    Number::exact(Rational::int(n), false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(word: &str) -> Entity {
        let hits = lookup(word);
        assert_eq!(hits.len(), 1, "{word}: {hits:?}");
        hits[0].entity
    }

    fn prop(word: &str) -> Prop {
        lookup_prop(word)[0]
    }

    #[test]
    fn counts() {
        let by = |k: Kind| entities().filter(|e| e.kind() == k).count();
        assert_eq!(by(Kind::Constant), 355);
        assert_eq!(by(Kind::Element), 118);
        assert_eq!(by(Kind::Planet), 8);
        assert_eq!(by(Kind::Moon), 21);
    }

    #[test]
    fn every_value_parses() {
        for e in entities() {
            assert_eq!(e.values().count(), e.def().props.len(), "{}", e.display());
        }
    }

    #[test]
    fn earth() {
        let earth = one("Earth");
        assert_eq!(earth.kind(), Kind::Planet);
        let m = earth.value(prop("mass")).unwrap();
        assert_eq!(m.unit.display(false), "kg");
        assert!((m.num.to_f64() - 5.9722e24).abs() < 1e18);
        assert!(m.source.contains("earthfact.html"));
        assert_eq!(
            earth.resolve_prop(&lookup_prop("distance")),
            Some(prop("distance from sun"))
        );
    }

    #[test]
    fn mercury_is_two_things() {
        let kinds: Vec<Kind> = lookup("mercury").iter().map(|h| h.entity.kind()).collect();
        assert!(kinds.contains(&Kind::Planet) && kinds.contains(&Kind::Element));
        assert_eq!(one("Hg").kind(), Kind::Element);
    }

    #[test]
    fn element_symbols_that_are_units_are_shadowed() {
        let w: Vec<Hit> = lookup("W");
        assert_eq!(w.len(), 1);
        assert!(w[0].shadowed, "W is the watt");
        assert!(!lookup("tungsten")[0].shadowed);
        assert!(!lookup("wolfram")[0].shadowed);
        assert!(!lookup("Au")[0].shadowed);
    }

    #[test]
    fn constants() {
        let g = one("G");
        assert_eq!(g.kind(), Kind::Constant);
        let v = g.value(g.default().unwrap()).unwrap();
        assert_eq!(v.uncertainty, Some("0.00015e-11 m^3 kg^-1 s^-2"));
        let h: Vec<Hit> = lookup("h");
        assert!(h.iter().all(|x| x.shadowed), "h is the hour");
        assert!(!lookup("planck constant")[0].shadowed);
    }

    #[test]
    fn digits() {
        assert_eq!(significant_digits("6.67430e-11 m^3 kg^-1 s^-2"), Some(6));
        assert_eq!(significant_digits("0.0012 g"), Some(2));
        assert_eq!(significant_digits("8611 m"), Some(4));
        assert_eq!(significant_digits("330000 lb"), None);
        assert_eq!(significant_digits("-4.664345550e-4"), Some(10));
        assert_eq!(significant_digits("c yr"), None);
    }

    #[test]
    fn facts() {
        let w = one("tungsten");
        assert!(w.fact("named").unwrap().text.contains("wolf rahm"));
        assert_eq!(w.fact("state").unwrap().text, "solid");
    }
}
