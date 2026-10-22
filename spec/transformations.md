# Meter transformations — الزحاف والعِلَل

Named transformation definitions and original-letter mechanics for `watad-tahwil`
and profile recipes. These belong to the meter layer; Arabic spelling conversion
is a separate concern.

## 0. Sources and scope

- Named definitions are transcribed from **الكافي في العروض والقافية (التبريزي)**, the chapter
  «الباب الأول في بيانِ ألقاب الزحاف والعِلَل وتعاريفها», as reproduced in two independent texts
  [S3] [S4] (verbatim definitions in §2–§4).
- Units, feet and beat words use [S1], as specified in `spec/circles.md`.
- Meter-specific permissions are documented in `spec/profiles/classical-v1.md`.
  Mechanical applicability does not grant a historical licence.
- Every rule below carries its source quote. Anything not verbatim is marked (paraphrase).

## 1. What may change

Definition [S3][S4]:

> الزِّحاف: تغيير مختصٌّ بثواني الأسباب مطلقًا بلا لزوم، ولا يدخل الأول والثالث والسادس من الجزء.

- **زحاف** = a change to a foot, at the _seconds of the sababs_ only, **without لزوم**: if it occurs
  in one verse it need not occur in the rest.
- Letter positions 1, 3 and 6 of a foot never change. The named زحافات act at positions **2, 4, 5
  and 7** (§2).
- **عِلَّة** = «ما إذا عرض لزم» [S3]: it occurs only at the **عروض والضرب** (last foot of each
  hemistich), never in the حشو — **except الخَرْم** (§4.3) — and once used it is **binding** across
  the poem.

Positions are counted in the foot's letters (الكتابة العروضية), which is the concatenation of its
unit letters — the same letter stream as the structural unit words. A letter is sākin (`0`) or mutaharrik
(`/`) per the foot's beat word.

## 2. Simple زحافات — المفرد ثمانية [S3][S4]

| Name      | Letter | Precondition | Operation | Source example                              |
| --------- | ------ | ------------ | --------- | ------------------------------------------- |
| الخَبْن   | 2      | sākin        | delete    | مستفعلن → متفعلن                            |
| الإِضْمار | 2      | mutaharrik   | silence   | متفاعلن → مُتْفَاعِلُن (pattern of مستفعلن) |
| الوَقْص   | 2      | mutaharrik   | delete    | متفاعلن → مُفَاعِلُن (pattern of مفاعلن)    |
| الطَّيّ   | 4      | sākin        | delete    | مستفعلن → مُسْتَعِلُنْ (مستعلن)             |
| القَبْض   | 5      | sākin        | delete    | مفاعيلن → مفاعلن؛ فعولن → فعول              |
| العَصْب   | 5      | mutaharrik   | silence   | مفاعلتن → (pattern of مفاعيلن)              |
| العَقْل   | 5      | mutaharrik   | delete    | مفاعلتن → (pattern of مفاعلن)               |
| الكَفّ    | 7      | sākin        | delete    | فاعلاتن → فاعلاتُ                           |

Source quotes:

- «الخبن: وهو حذف ثاني الجزء ساكنًا. والإضمار: إسكانه متحركًا. والوقْص: حذفه متحركًا. والطيُّ:
  حذف رابعه ساكنًا. والقبض: حذف خامسه ساكنًا. والعصْب: إسكانه بعد تَحرُّكه. والعَقْل: حذفه
  متحركًا وهذان لا يكونان إلا في مفاعلتن فيبدل في الأول مفاعيلن، وفي الثاني مفاعلن. والكفُّ:
  حذفُ سابعه ساكنًا.» [S3][S4]
- The named examples are attested forms from [S5] and standard references. All eight were checked
  mechanically against the §6 rules on the layer-1 letter streams (letter position + precondition +
  operation reproduce the stated result word and its beats). Independent named-example tests check this correspondence.
- **Restriction (verbatim):** العَصْب والعَقْل «لا يكونان إلا في مفاعلتن» [S3][S4].

## 3. Compound زحافات — المزدوج أربعة [S3][S4]

| Name     | Components     | Restriction                                     | Result                                       |
| -------- | -------------- | ----------------------------------------------- | -------------------------------------------- |
| الخَبْل  | الطي + الخبن   | —                                               | —                                            |
| الخَزْل  | الطي + الإضمار | متفاعلن only                                    | مُفْتَعِلن («إسكان (تا) وحذف (ألف) متفاعلن») |
| الشَّكْل | الكف + الخبن   | فاعلاتن (مجموع الوتد) و مستفعِ لن (مفروق الوتد) | «ينقلان إلى فاعلن ومفاعل»                    |
| النَّقْص | الكف + العصب   | مفاعلتن only                                    | مفاعيل                                       |

Source quote: «وأمَّا المزْدَوجُ فأربعةٌ: الطيُّ مع الخَبْن خَبْلٌ، ومع الإضمار خَزْلٌ، ويُحصَر في
إسكان (تا) وحذف (ألف) متفاعلن فيبدل إلى مُفْتَعِلن، والكفُّ مع الخبن شكْلٌ وانحصر في حذف الألف
الأولى والنون من (فاعلاتن) مجموع الوتد، والسين والنون من (مستفع لن) مفروق الوتد فينقلان إلى
فاعلن ومفاعل ومع العصب نقصٌ، ويدخل مفاعلتن فقط فينقل إلى مفاعيل.» [S3][S4]

## 4. العِلَل

### 4.0 Definition and scope

> وأمَّا العِلَلُ فهي: ما إذا عرض لزم، ولا تكون في الحشو بل في الضرب والعروض، ما عدا الخَرْم [S3]

- Applied only at العروض/الضرب; **binding** (لزوم) once used in the poem.
- Exception: الخَرْم (§4.3).

### 4.1 علل الزيادة

| Name       | Operation                                        | Where (source)                 | Named result                                           |
| ---------- | ------------------------------------------------ | ------------------------------ | ------------------------------------------------------ |
| التَّرْفيل | add a sabab khafīf to a foot ending in وتد مجموع | مجزوء المتدارك والكامل         | فاعلن → فاعلاتن؛ متفاعلن → متفاعلاتن                   |
| التَّذْييل | add a sākin letter to a foot ending in وتد مجموع | مجزوء الكامل، البسيط، المتدارك | متفاعلن → متفاعلان؛ مستفعلن → مستفعلان؛ فاعلن → فاعلان |
| التَّسْبيغ | [add] to a foot ending in sabab khafīf           | مجزوء الرمل                    | فاعلاتن → فاعلاتان                                     |

Source quote: «فزيادةُ سَبَبٍ خَفيفٍ على ما آخره وتد مجموع: ترفيلٌ، ولا يقع إلا في مجزوء
المتدارك والكامل فيصير فاعلن في الأول (فاعلاتن)، ومتفاعلنٌ في الثاني (متفاعلاتن). وحرفٌ ساكنٌ
على ما آخره وتد مجموع تذييلٌ وهو خاصٌّ بمجزوء الكامل والبسيط والمتدارك فيصير متفاعلن في الأول
(متفاعلان)، ومستفعلن في الثاني (مستفعلان)، وفاعلن في الثالث (فاعلان). وعلى [ما] آخره سبب
خفيف: تَسبيغٌ وهو خاصٌّ بمجزوء الرمل فيصير فاعلاتن فيه (فاعلاتان)، وخُصّتْ علل الزيادة بالمجزوء
عوضًا عن النقْصِ.» [S3][S4]

Note (verbatim): «خُصّتْ علل الزيادة بالمجزوء عوضًا عن النقْصِ» — the زيادة علل specialize to
مجزوء meters.

### 4.2 علل النقصان

| Name     | Operation                               | Where (source)                             | Named result                                                                    |
| -------- | --------------------------------------- | ------------------------------------------ | ------------------------------------------------------------------------------- |
| الحَذْف  | «ذهاب سبب خفيف»                         | —                                          | —                                                                               |
| القَطْف  | الحذف مع العصب                          | الوافر                                     | مفاعلتن → مفاعل، «وينقل إلى فعولن»                                              |
| القَطْع  | «حذف ساكن الوتد المجموع وإسكان ما قبله» | البسيط، الكامل، الرجز                      | فاعلن → فاعلْ؛ متفاعلن → متفاعلْ؛ مستفعلن → مستفعلْ («بإسكان اللام في الثلاثة») |
| البَتْر  | القطع مع الحذف                          | «يدخل — كما قال الخليل — المتقارب والمديد» | —                                                                               |
| القَصْر  | «حذف ساكن السبب وإسكان متحركه»          | —                                          | —                                                                               |
| الحَذَذ  | «حذف وتد مجموع»                         | الكامل                                     | «يحذف (علن) من متفاعلن وينقل الباقي إلى فَعِلن»                                 |
| الصَّلْم | «ومفروقٍ صَلْمٌ» [حذف وتد مفروق]        | السريع                                     | «تحذف (لات) من مفعولات وينقل الباقي إلى فَعْلن»                                 |
| الوَقْف  | «إسكان السابع المتحرك»                  | —                                          | —                                                                               |
| الكَسْف  | «وحذفه» [السابع المتحرك]                | —                                          | «وليس لنا سابع متحركٍ إلا (تاء) مفعولات»                                        |

Source quotes: «وأمَّا علل النقصان فذهاب سبب خفيفٍ حَذْفٌ، وهو مع العَصْب قَطْفٌ وهو خاصٌّ
بالوافر فيصير مفاعلتن فيه (مفاعل) ويُنْقل إلى (فعولن) وحذف ساكن الوتد المجموع وإسكان ما قبله
قطْعٌ ويختصُّ بالبسيط والكامل والرَّجز، فيصير فاعلن في الأول ومتفاعلن في الثاني ومستفعلن في
الثالث (فاعلْ) و (متفاعلْ) و (مستفعلْ) بإسكان اللام في الثلاثة. وهو مع الحذف بَتْرٌ ويدخل ــ
كما قال الخليل ــ المتقارب والمديد» [S3]؛ «وحَذْفُ ساكنِ السبب وإسكان متحرِّكه قَصْرٌ، وحَذْفُ
وَتَدٍ مجموعٍ حَذَذٌ، ولا يدخل إلا الكامل فيحذف (علن) من متفاعلن وينقل الباقي إلى (فَعِلن)،
ومفروقٍ صَلْمٌ ولا يدخل إلا السريع الذي أجزاؤه مستفعلن مستفعلن مفعولات فتحذف (لات) وينقل الباقي
إلى (فَعْلن). وإسكان السابع المتحرك وقْفٌ وحذفه كسْفٌ، وليس لنا سابع متحركٍ إلا (تاء) مفعولات.»
[S3][S4] (tail; §7 notes the confidence level).

### 4.3 الخَرْم — the exception

- الخَرْم is the only علة that may occur in the **حشو** («ما عدا الخَرْم» [S3]).
- Kharm deletes the first moving letter of an **initial majmu watad**, at the
  beginning of the verse. [S6], خاتمة, printed pp. 344–345, supplies the definition
  and names الطويل والمتقارب والوافر والهزج والمضارع. It is not a licence to
  alter arbitrary body feet. A single-foot hemistich retains its initial flag
  independently of its terminal flag. Whether Kharm is licensed in a given
  hemistich/context remains explicit profile data.

### 4.4 Structural recipe semantics

[S7], introduction §7 and the table of علل النقص, confirms Qasr as deleting the
final khafif sabab's still letter and silencing its moving letter. Batr removes
the final khafif sabab and applies Qat to the immediately preceding majmu watad;
it is not a simultaneous application of Qat to the original final sabab. Both
recipes require their stated structural units. The original-word program records
all resulting edits against the original base, avoiding shifted coordinates.
Kharm, Qasr and Batr now have mechanical implementations; this does not change an
unknown or forbidden permission into an allowed permission in any source profile.

## 5. Reading "ينقل إلى / يُبدل إلى X"

When a source says a derived form «ينقل إلى» or «يُبدل إلى» a named foot (e.g. مفاعلتن with العصب
→ مفاعيلن), the statement is about the **beat word**: the derived foot's beat word equals the named
canonical foot's beat word. The name of a derived form is the canonical foot whose word matches it.

## 6. Machine mapping for `watad-tahwil`

- A foot is its **letter stream** = concatenation of its units' letters (layer-1 words). Each letter
  has a beat: `/` or `0` (layer-1 beat words).
- A **simple زحاف** = `(position, precondition, operation)` with `position ∈ {2, 4, 5, 7}`
  (1-based, letters), `precondition ∈ {sākin, mutaharrik}`, `operation ∈ {delete, silence}` — §2.
  A named زحاف is applicable iff the base foot's letter at `position` meets the precondition.
- A **compound زحاف** = a fixed pair of simple ones (§3) with its own name and stated restriction.
- An **عِلَّة** = a change at the last foot (عروض/ضرب) only, binding across the poem; the §4 tables
  give operation + specialisation; per-meter permissions come from the selected profile.
- The **derived-name tests**: for every attested (foot, operation, position) the
  classical name and the resulting beat word must match the source examples above.

## 7. Scope and source limitations

### 7.1 Profile permissions

Named-example tables are not exhaustive per-meter permission matrices.
Consult the selected source profile for supported layouts and licences.

### 7.2 Context and source conventions

Cross-hemistich mu'aqabah, competing endpoints and different schools require
explicit source conventions. Supported and unsupported readings are listed
in the profile inventory.

### 7.3 Shakl surface correspondence

For الشَّكْل on فاعلاتن, deleting original letters 2 and 7 leaves `///0/`.
[S3][S4] give a derived-name wording that does not match that mechanical
surface. The source wording is retained separately from the computed result;
the API does not infer a historical licence from this name correspondence.

## Sources

- **[S1]** أحمد الهاشمي، «ميزان الذهب في صناعة شعر العرب»، مقدمات علم العروض:
  <https://downloads.hindawi.org/books/50259706.pdf>.
- **[S2]** محمد الصاعدي، «أهدي سبيل إلى علمي الخليل»، الجزيرة، 2024-02-02:
  `https://www.al-jazirah.com/2024/20240202/cm21.htm` — the six مهملة.
- **[S3]** «الباب الأول في بيانِ ألقاب الزحاف والعِلَل وتعاريفها» (الكافي، التبريزي) as reproduced
  in عبد الرحمن بن يحيى المعلمي اليماني، «آثار» — `https://shamela.ws/book/14381/8957`.
- **[S4]** Same chapter, independent reproduction:
  `https://www.fatihsyuhud.org/2022/09/bab-awwal-alkafi.html`.
- **[S5]** «ميزان الذهب»، definitions and named examples of simple zihaf:
  <https://www.safahat.org/books/50259706/1.1/>.
- **[S6]** المعلمي، مختصر متن الكافي في العروض والقوافي، خاتمة، pp. 344–345:
  <https://usul.ai/ar/t/mukhtasar-matn-al-kafi-fi-al-arud-wal-qawafi>.
- **[S7]** أحمد الهاشمي، ميزان الذهب، مقدمات علم العروض، §7 and table of علل النقص:
  <https://www.safahat.org/books/50259706/1.1/>.
