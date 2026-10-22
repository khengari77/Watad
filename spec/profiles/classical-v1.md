# Classical meter profile inventory, revision 1

This inventory distinguishes generated circles from linguistic usage. The
mathematical engine checks any declared finite structured profile; the built-in
catalog records a versioned reading of the sources below. It does not equate
all mechanically possible edits with attested Arabic poetry.

## Identity and authority

- Circle IDs: family `1000 + MeterName::id()`, revision 1. All 22 rotations use
  their complete circle-derived foot sequence without transformations.
- Usage IDs: family `2000 + MeterName::id()`, revision 1. All sixteen used meters
  have explicit retained-foot layouts, terminal pairs and body alternatives.
- The `1:1` restricted Hazaj test fixture describes a separate, narrower
  language. An ID never substitutes for an externally supplied definition.
- The six unused rotations have circle scope, with no invented usage language.

Primary terminal/usage authority: al-Hashimi, *Mizan al-Dhahab*, Hindawi 2016,
[printed edition](https://downloads.hindawi.org/books/50259706.pdf), with a
chapter transcription at
[Mizan al-Dhahab, numbered meter chapters and footnotes](https://www.safahat.org/books/50259706/1.2/).
Mechanical definitions and general operation listings are in its
[introduction, §§4–7](https://www.safahat.org/books/50259706/1.1/).
Initial Kharm is checked against al-Mu'allimi's
[Mukhtasar Matn al-Kafi, conclusion, printed pp. 344–345](https://usul.ai/ar/t/mukhtasar-matn-al-kafi-fi-al-arud-wal-qawafi).
The within-hemistich mu'aqabah convention is clarified by al-Zamakhshari's
[al-Qistas, foot-variation discussion](https://www.arabic-keyboard.info/books/ar/read-book/%D8%A7%D9%84%D9%82%D8%B3%D8%B7%D8%A7%D8%B3-%D9%81%D9%8A-%D8%B9%D9%84%D9%85-%D8%A7%D9%84%D8%B9%D8%B1%D9%88%D8%B6).
These are source choices, rather than facts proved by Verus.

## Layout and terminal matrix

`N` means unchanged. Operations on a line below act on that layout's original
last foot; they are not substitutions of a same-surface foot with different
ownership. Separate rows/groups retain distinct binding classes unless a source
explicitly makes the variation optional.

| Meter / chapter | Feet per hemistich | Arud → darb alternatives |
| --- | --- | --- |
| الطويل / 1 | 4 | Qabd → N, Qabd, Hadhf |
| المديد / 2 | 3 | N → N; Hadhf → Qasr, Hadhf, Batr; Khabn+Hadhf → Khabn+Hadhf, Batr |
| البسيط / 3 | 4 / 3 | Full: Khabn → Khabn, Qat with supplied ridf evidence. Short: N → Tadhyil, N, Qat; Qat → Qat |
| الوافر / 4 | 3 / 2 | Full: Qatf → Qatf. Short: N → N, Asb; optional Asb arud requires an unaltered arud witness somewhere in the poem |
| الكامل / 5 | 3 / 2 | Full: N → N, Qat, Idmar+Hadhadh; Hadhadh → Hadhadh, Idmar+Hadhadh. Short: N → Tarfil, Tadhyil, N, Qat |
| الهزج / 6 | 2 | N (optional arud Kaf) → N, Hadhf |
| الرجز / 7 | 3 / 2 | Full: N → N, Qat. Short: N → N |
| الرمل / 8 | 3 / 2 | Full: Hadhf → N, Qasr, Hadhf. Short: N → Tasbigh, N, Hadhf |
| السريع / 9 | 3 | Tayy+Kasf → Tayy+Waqf, Tayy+Kasf, Salm; Khabn+Tayy+Kasf → same, Salm |
| المنسرح / 10 | 3 | N → Tayy, Qat; Tayy → Tayy |
| الخفيف / 11 | 3 / 2 | Full: N → N (optional Tashith), Hadhf; Hadhf → Hadhf. Short: N → N, Khabn+Qasr |
| المضارع / 12 | 2 | N (optional arud Kaf) → N; first Mafailun must carry Qabd or Kaf |
| المقتضب / 13 | 2 | Tayy → Tayy; initial MafUlatu has one of Khabn/Tayy, never both or neither |
| المجتث / 14 | 2 | N → N with optional Tashith |
| المتقارب / 15 | 4 / 3 | Full: N → N, Qasr, Hadhf, Batr; ordinary full arud also admits Qabd/Hadhf as nonbinding variants. Short: Hadhf → Hadhf, Batr |
| المتدارك / 16 | 4 / 3 | Full: N → N. Short: N → N, Khabn+Tarfil, Tadhyil |

Explicit single-line branches represent mashṭur/manhuk Rajaz, mashṭur Sari
(Waqf/Kasf), and manhuk Munsarih (Waqf/Kasf). Their ajuz slot sequence, external
ajuz word and ajuz certificate are all empty. The line is never duplicated to
fabricate a verse. Both initial and terminal flags apply to a one-foot layout.

## Variation and binding conventions

Body alternatives are explicit recipe sets in `catalog::body`, not inferred from
a transformation's structural applicability. The catalog includes ordinary
single zihaf and declared compounds, initial Kharm in the five named meters,
Idmar endpoint variants in Kamil, Khabn endpoint variants where declared, and
nonbinding Qat in Mutadarak. Each zihaf component retains its original sabab
ownership check even when composed with an illa.

Madid, Ramal, Khafif and Mujtath use explicit within-hemistich mu'aqabah exclusions:
Kaf of the preceding foot cannot coexist with Khabn of the following foot.
Source expansion produces correlated branches; the checked lowering boundary
independently checks *every* alternative pair against the declared original-letter
exclusion. Branch expansion cannot silently bypass the restriction.

For Mujtath, al-Qistas additionally conditions Kaf of Failatun on preservation
of the preceding MustafIlunMafruq sin (original letter two). The strict profile
checks both correlations and allows Kaf/Shakl at the arud, while retaining
N/Khabn/Tashith at the darb. Al-Sakkaki, *Miftah al-Ulum*, ed. Nu'aym Zarzur, 2nd ed. 1987,
[printed p. 559](https://archive.org/download/manizheteimurie_yahoo/page/n564.jpg),
explicitly excludes Kaf/Shakl at the darb. Al-Hashimi p. 87 note 12 gives
broader Shakl wording; that competing reading is outside this selected profile.
Al-Qistas, ed. Fakhr al-Din Qabawa, 2nd rev. ed. 1989,
[printed p. 123](https://archive.org/download/AAlexandrina-108959/page/n124.jpg),
records the reciprocal constraints and disagreement following the sound-sin
condition. These are versioned conventions, not universal historical prohibitions.

Optional Tashith shares its ordinary binding class. Wafir's optional Asb arud
shares its ordinary class, and its existence requirement is evaluated over the
whole poem (al-Hashimi p. 49 note 5: «بشرط أن تبقى صحيحة على الأقل مرة واحدة»).
A sound body or darb does not replace the arud witness. Full Wafir has a
separate class; its Qatf forms do not rescue an incompatible shortened poem.
The source note does not separately adjudicate tasri witness status.
Recognition searches complete assignments; it cannot commit greedily
to the first reading of an ambiguous verse.

Context 0 denotes the ordinary reading. Context 1 denotes tasri and is initial
verse only. Basit's conditional cut darb requires context 2 (supplied ridf
fact); context 3 combines that fact with initial tasri. These are external
context claims. Beat words do not establish rhyme identity, rhyme letters or
whether a real Arabic text satisfies the supplied claim. A poem supplies its
context separately for every verse.

## Coverage decisions and unresolved readings

Strict acceptance uses only the declared data. Unknown/unimplemented readings
are not permissive defaults. The following need independent source profiles or
further source adjudication before broad historical-coverage claims:

| Reading | Status in this catalog |
| --- | --- |
| Cross-hemistich mu'aqabah | Unknown for this selected convention; al-Qistas explicitly records disagreement. Not imposed as an invented universal rule. |
| Full treatment of metrical taste (حسن/صالح/قبيح) | Source commentary; no binary rejection solely because an allowed form is called ugly. |
| Conflicting general vs local compound licences | Conservative local reading. In particular Madid's local ban on Khabn+Kaf overrides the general introductory list. Further readings require named profiles. |
| Rajaz's changing-rhyme, repeatedly rhymed-couplet convention | Not claimed by the ordinary fixed-class profile; rhyme evidence is outside beat-only input. |
| Every historical Khabab school | Not claimed by this Mutadarak reading; declared Khabn/nonbinding Qat are supported. |
| Other rare forms and competing terminal classifications | Unknown until source data is transcribed and reviewed. No usage grammar is invented for unused rotations. |

The engine is exact for the declared profiles. This inventory deliberately does
not claim that the built-in source data exhausts every Arabic school or licence.
The mathematical engine's completeness is relative to declared profiles,
separate from historical completeness.

## Evidence

`source_examples.rs` independently enters canonical beat patterns for every used
meter. `catalog.rs` tests compile and replay each declared branch. Additional
regressions reject missing obligatory changes, misplaced initial contexts, absent
Wafir witnesses, invalid adjacency alternatives, wrong meter/profile metadata,
gaps/overlaps and extra input. Arbitrary-language and poem-assignment properties
compare exact recognition against independent exhaustive oracles.
