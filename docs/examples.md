# Narrative Sentence Splitting Examples

## Dialog spanning paragraph separators

**Input text:**
```
A bit of a scare shot through Tom—a touch of uncomfortable suspicion. He
searched Aunt Polly's face, but it told him nothing. So he said:

"No'm—well, not very much."

The old lady reached out her hand and felt Tom's shirt, and said:

"But you ain't too warm now, though." And it flattered her to reflect
that she had discovered that the shirt was dry without anybody knowing
that that was what she had in her mind. But in spite of her, Tom knew
where the wind lay, now. So he forestalled what might be the next move:

"Some of us pumped on our heads—mine's damp yet. See?"
```

**Output:**
```
91    A bit of a scare shot through Tom—a touch of uncomfortable suspicion.    (578,1,578,70)
92    He searched Aunt Polly's face, but it told him nothing.    (578,71,579,53)
93    So he said: "No'm—well, not very much."    (579,54,581,28)
94    The old lady reached out her hand and felt Tom's shirt, and said: "But you ain't too warm now, though."    (583,1,585,38)
95    And it flattered her to reflect that she had discovered that the shirt was dry without anybody knowing that that was what she had in her mind.    (585,39,587,40)
96    But in spite of her, Tom knew where the wind lay, now.    (587,41,588,25)
97    So he forestalled what might be the next move: "Some of us pumped on our heads—mine's damp yet. See?"    (588,26,590,55)
```

## Paragraph separators indicating end of never-closed quote

**Input text:**
```
"Come, Victor; not brooding thoughts of vengeance against the assassin,
but with feelings of peace and gentleness, that will heal, instead of
festering, the wounds of our minds. Enter the house of mourning, my
friend, but with kindness and affection for those who love you, and not
with hatred for your enemies.

"Your affectionate and afflicted father,

"Alphonse Frankenstein.
```

**Output:**
```
673    "Come, Victor; not brooding thoughts of vengeance against the assassin, but with feelings of peace and gentleness, that will heal, instead of festering, the wounds of our minds. Enter the house of mourning, my friend, but with kindness and affection for those who love you, and not with hatred for your enemies.    (2064,1,2068,30)
674    "Your affectionate and afflicted father, "Alphonse Frankenstein.    (2070,1,2072,24)
```