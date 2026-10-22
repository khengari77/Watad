# دوائر الخليل — Circles, feet and meters

This is the data spec for `watad-types` and `watad-daira`. Every table below is checked mechanically by the
`const` assertions in `watad-daira` (theorem numbers in §6). Source tags point at §7.

Terminology: `/` = متحرك (mutaḥarrik), `0` = ساكن (sākin). A _unit word_ is a sequence of sabab/watad units;
a _rotation_ of a circle word is a cyclic shift; a meter is a rotation, and its hemistich is that rotation
repeated a whole number of times.

## 1. Units

| Unit      | Beats            | Pattern | Example |             |
| --------- | ---------------- | ------- | ------- | ----------- |
| سبب خفيف  | متحرك ساكن       | `/0`    | لَمْ    | SababKhafif |
| سبب ثقيل  | متحرك متحرك      | `//`    | لَكَ    | SababThaqil |
| وتد مجموع | متحرك متحرك ساكن | `//0`   | نَعَمْ  | WatadMajmu  |
| وتد مفروق | متحرك ساكن متحرك | `/0/`   | قَالَ   | WatadMafruq |

Source: [S1], same convention as «ميزان الذهب». Examples are the classical ones.

## 2. The tafʿīlāt

The 10 canonical feet, plus one foot attested only in المتوافر (§5 note). Each foot has exactly **one**
watad — the invariant the type layer enforces (a watad is never altered by زحاف in the حشو).

| Foot      | Units | Pattern    |                                                        |
| --------- | ----- | ---------- | ------------------------------------------------------ |
| فعولن     | W S   | `//0/0`    | FaUlun                                                 |
| فاعلن     | S W   | `/0//0`    | FaIlun                                                 |
| مفاعيلن   | W S S | `//0/0/0`  | Mafailun                                               |
| مستفعلن   | S S W | `/0/0//0`  | MustafIlun                                             |
| فاعلاتن   | S W S | `/0//0/0`  | Failatun                                               |
| مفاعلتن   | W T S | `//0///0`  | Mafalatun                                              |
| متفاعلن   | T S W | `///0//0`  | MutaFailun                                             |
| مفعولاتُ  | S S M | `/0/0/0/`  | MafUlatu                                               |
| فاعِ لاتن | M S S | `/0//0/0` | FailatunMafruq                                         |
| مستفعِ لن | S M S | `/0/0//0`  | MustafIlunMafruq                                       |
| فاعلاتُكَ | S W T | `/0//0//`  | Failatuka — attested only as the foot of المتوافر [S2] |

Notes:

- The two mafrūq forms share their surface pattern with a canonical foot (مستفعِ لن = مستفعلن on the
  surface, فاعِ لاتن = فاعلاتن on the surface) but differ in **structure** (where the watad sits), and
  they take different زحافات. Structure over strings is a hard requirement of the type layer.
- فاعلاتُكَ is not among the classical ten; it appears as the تفعيلة of المتوافر [S2]. It is the only
  attested foot ending in a sabab thaqīl, and it is why المتوافر's rotation does not decompose into the
  canonical ten.

## 3. The five circles

Each circle is a cyclic word over units; its primitive period is the word length (verified: theorem 1).
Meters are listed **in circle order** (as they fall on the circle), which is the rotation offset.

| Circle | Word              | Pattern        | Period | Meters in circle order                                                              |
| ------ | ----------------- | -------------- | ------ | ----------------------------------------------------------------------------------- |
| مختلف  | W S W S S         | `//0/0//0/0/0` | 5      | الطويل، المديد، المستطيل، البسيط، الممتد [S3]                                       |
| مؤتلف  | W T S             | `//0///0`      | 3      | الوافر، الكامل، المتوافر [S3]                                                       |
| مجتلب  | W S S             | `//0/0/0`      | 3      | الهزج، الرجز، الرمل [S3]                                                            |
| مشتبه  | S S W S S W S S M | `…/0/`         | 9      | السريع، المتئد، المنسرد، المنسرح، الخفيف، المضارع، المقتضب، المجتث، المطرد [S3, S2] |
| متفق   | W S               | `//0/0`        | 2      | المتقارب، المتدارك [S3]                                                             |

- 5 circles × their periods = 5+3+3+9+2 = **22 meters**: 16 مستعمل + 6 مهمل.
- Named for their foot variety: مختلف (mixed 5- and 7-letter feet), مؤتلف (uniform), مجتلب, مشتبه, متفق.
- **المتدارك** is مهمل at al-Khalīl and was made a used meter by al-Akhfash [S3]. The circle derives the
  pattern either way; the "used" label is the Akhfash convention, and the project states this explicitly.

## 4. The 22 rotations

Rotation _k_ = cyclic shift by _k_ units of the circle word. Pattern = beats of the rotation. Feet are the
**دائرة (circle) form** of the hemistich, not the common usage form (see §5 notes). Rows are the
authoritative table; `watad-daira` regenerates and checks every cell (theorems 1–5).

### مختلف (period 5, 2 reps per hemistich = 10 units)

| rot | Meter    | Status | Word      | Pattern        | Feet (دائرة)                     |
| --- | -------- | ------ | --------- | -------------- | -------------------------------- |
| 0   | الطويل   | مستعمل | W S W S S | `//0/0//0/0/0` | فعولن مفاعيلن فعولن مفاعيلن      |
| 1   | المديد   | مستعمل | S W S S W | `/0//0/0/0//0` | فاعلاتن فاعلن فاعلاتن فاعلن      |
| 2   | المستطيل | مهمل   | W S S W S | `//0/0/0//0/0` | مفاعيلن فعولن مفاعيلن فعولن [S2] |
| 3   | البسيط   | مستعمل | S S W S W | `/0/0//0/0//0`  | مستفعلن فاعلن مستفعلن فاعلن      |
| 4   | الممتد   | مهمل   | S W S W S | `/0//0/0//0/0`  | فاعلن فاعلاتن فاعلن فاعلاتن [S2] |

### مؤتلف (period 3, 3 reps per hemistich = 9 units)

| rot | Meter    | Status | Word  | Pattern   | Feet (دائرة)                       |
| --- | -------- | ------ | ----- | --------- | ---------------------------------- |
| 0   | الوافر   | مستعمل | W T S | `//0///0` | مفاعلتن مفاعلتن مفاعلتن            |
| 1   | الكامل   | مستعمل | T S W | `///0//0` | متفاعلن متفاعلن متفاعلن            |
| 2   | المتوافر | مهمل   | S W T | `/0//0//` | فاعلاتُكَ فاعلاتُكَ فاعلاتُكَ [S2] |

### مجتلب (period 3, 3 reps per hemistich = 9 units)

| rot | Meter | Status | Word  | Pattern   | Feet (دائرة)            |
| --- | ----- | ------ | ----- | --------- | ----------------------- |
| 0   | الهزج | مستعمل | W S S | `//0/0/0` | مفاعيلن مفاعيلن مفاعيلن |
| 1   | الرجز | مستعمل | S S W | `/0/0//0` | مستفعلن مستفعلن مستفعلن |
| 2   | الرمل | مستعمل | S W S | `/0//0/0` | فاعلاتن فاعلاتن فاعلاتن |

### مشتبه (period 9, 1 rep per hemistich = 9 units)

| rot | Meter   | Status | Word              | Pattern                   | Feet (دائرة)                   |
| --- | ------- | ------ | ----------------- | ------------------------- | ------------------------------ |
| 0   | السريع  | مستعمل | S S W S S W S S M | `/0/0//0/0/0//0/0/0/0/`   | مستفعلن مستفعلن مفعولاتُ       |
| 1   | المتئد  | مهمل   | S W S S W S S M S | `/0//0/0/0//0/0/0/0//0`   | فاعلاتن فاعلاتن مستفعِ لن [S2] |
| 2   | المنسرد | مهمل   | W S S W S S M S S | `//0/0/0//0/0/0/0//0/0`   | مفاعيلن مفاعيلن فاعِ لاتن [S2] |
| 3   | المنسرح | مستعمل | S S W S S M S S W | `/0/0//0/0/0/0//0/0//0`    | مستفعلن مفعولاتُ مستفعلن       |
| 4   | الخفيف  | مستعمل | S W S S M S S W S | `/0//0/0/0/0//0/0//0/0`    | فاعلاتن مستفعِ لن فاعلاتن      |
| 5   | المضارع | مستعمل | W S S M S S W S S | `//0/0/0/0//0/0//0/0/0`    | مفاعيلن فاعِ لاتن مفاعيلن      |
| 6   | المقتضب | مستعمل | S S M S S W S S W | `/0/0/0//0/0//0/0/0//0`   | مفعولاتُ مستفعلن مستفعلن       |
| 7   | المجتث  | مستعمل | S M S S W S S W S | `/0/0//0/0//0/0/0//0/0` | مستفعِ لن فاعلاتن فاعلاتن      |
| 8   | المطرد  | مهمل   | M S S W S S W S S | `/0//0/0//0/0/0//0/0/0`  | فاعِ لاتن مفاعيلن مفاعيلن [S2] |

### متفق (period 2, 4 reps per hemistich = 8 units)

| rot | Meter    | Status              | Word | Pattern | Feet (دائرة)            |
| --- | -------- | ------------------- | ---- | ------- | ----------------------- |
| 0   | المتقارب | مستعمل              | W S  | `//0/0` | فعولن فعولن فعولن فعولن |
| 1   | المتدارك | مستعمل (عند الأخفش) | S W  | `/0//0` | فاعلن فاعلن فاعلن فاعلن |

## 5. Notes on the table

- **الممتد's foot boundaries** are فاعلن فاعلاتن (S W | S W S), per [S2]. The unit word S W S W S also
  admits the boundary فاعلاتن فعولن (S W S | W S); the attested form wins, and the spec pins the
  attested boundaries.
- **Usage vs دائري form.** Circle forms preserve the complete repeated unit word. Usage may retain a shorter
  prefix (e.g. المديد، الهزج، المضارع، المقتضب، المجتث), and may apply an illa/zihaf at the
  عروض/ضرب — e.g. السريع in use ends فاعلن where the circle form ends مفعولاتُ [S1].
  The generated circle form and the source usage profile are separate APIs.
- **مهمل naming decision.** All six مهملة carry classical names: المستطيل، الممتد (different [S3]);
  المتوافر (مؤتلف) and المتئد، المنسرد، المطرد (مشتبه) [S2, S3]. Alternative names exist in secondary
  sources (الوسيط/الوسيم for المستطيل/الممتد; الغريب/القريب/المشاكل for المتئد/المنسرد/المطرد) but are
  not verified against a classical text — they are not used in code.
- **Hemistich lengths**: 10 units (مختلف), 9 (مؤتلف، مجتلب، مشتبه), 8 (متفق). The verse = صدر + عجز,
  both on the same feet; only the عروض/ضرب (last feet) may differ via declared transformations.

## 6. Theorems (checked at compile time in `watad-daira`)

1. **Primitivity.** Each circle word has primitive period = its length ⇒ exactly `period` distinct
   rotations per circle.
2. **Disjointness.** The 22 rotation words are pairwise distinct (across and within circles).
3. **Totality + injectivity.** The names table covers every `(circle, rot < period)` exactly once; all 22
   names are pairwise distinct. 22 = 16 مستعمل + 6 مهمل.
4. **Counts preserved.** A rotation preserves the multiset of units and the total number of beats of its
   circle word.
5. **Segmentation consistency.** For every meter: the listed feet concatenate to the rotation (a whole
   number of reps), every foot is one of the attested feet (§2), and every foot has exactly one watad.

The circle inventory is finite data; the theorems are exhaustive checks over it, not proofs about Arabic. They pin the
data to itself. The linguistic authority is §7.

## 7. Sources

- **[S1]** أحمد الهاشمي، «ميزان الذهب في صناعة شعر العرب»، مقدمات علم العروض:
  <https://downloads.hindawi.org/books/50259706.pdf>. The four units and canonical feet
  provide the structural vocabulary; usage permissions are documented separately.
- **[S2]** أ.د. عبدالرزاق الصاعدي، «البحور المهملة ودوائر الخليل»، جريدة الجزيرة، 2024-02-02 —
  <https://www.al-jazirah.com/2024/20240202/cm21.htm>: the six مهملة with their exact تفعيلات
  (المستطيل: مفاعيلن فعولن ×2؛ الممتد: فاعلن فاعلاتن فاعلن فاعلاتن؛ المتوافر: فاعلاتُكَ ×3؛ المتئد:
  فاعلاتن فاعلاتن مستفعِ لن؛ المنسرد: مفاعيلن مفاعيلن فاعِ لاتن؛ المطرد: فاعِ لاتن مفاعيلن مفاعيلن).
- **[S3]** «أهْدَى سبيل إلى علمي الخليل»، المكتبة الشاملة، ص89 —
  <https://ftp.shamela.ws/book/10860/129>: meters in circle order (المختلف: الطويل، المديد، المستطيل،
  البسيط، الممتد؛ المؤتلف: الوافر، الكامل، المتوافر؛ المجتلب: الهزج، الرجز، الرمل؛ المشتبه: السريع،
  مهمل، مهمل، المنسرح، الخفيف، المضارع، المقتضب، المجتث، مهمل؛ المتفق: المتقارب، المتدارك)، and
  المتدارك مهمل عند الخليل.
- **Mechanical check.** The rotation offsets were verified by exhaustive script check: each meter's foot
  concatenation equals exactly one rotation of exactly one circle word, all 16 مستعمل meters matched, and
  no rotation is claimed twice. The table is checked by `spec_sync` and compile-time assertions.
