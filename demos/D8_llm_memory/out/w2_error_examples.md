# W2 Error Analysis — Representative Examples

Source: `demos\D8_llm_memory\out\route_a_500q_results.json` (F-044 500Q paired outcomes)

## kdf_only (n=66)

### 6a1eabeb (knowledge-update, answer_type=short_fact)

- **Question**: What was my personal best time in the charity 5K run?
- **Ground truth**: `25 minutes and 50 seconds (or 25:50)`
- **KDF answer** (✅): Your personal best time in the charity 5K run was 25:50.
- **Mem0 answer** (❌): Your personal best time in the charity 5K run was 27:12.
- Mem0 retrieval recall: 0.000, haystack sessions: 2

### c4a1ceb8 (multi-session, answer_type=number)

- **Question**: How many different types of citrus fruits have I used in my cocktail recipes?
- **Ground truth**: `3`
- **KDF answer** (✅): You have used at least three different types of citrus fruits in your cocktail recipes: orange (in the Whiskey Sour and Sangria), lemon (in the Whiskey Sour and Cava Spritz), and lime (in the Daiquiri).
- **Mem0 answer** (❌): You have used lime and orange in your cocktail recipes.
- Mem0 retrieval recall: 0.000, haystack sessions: 4

### 7161e7e2 (single-session-assistant, answer_type=short_fact)

- **Question**: I'm checking our previous chat about the shift rotation sheet for GM social media agents. Can you remind me what was the rotation for Admon on a Sunday?
- **Ground truth**: `Admon was assigned to the 8 am - 4 pm (Day Shift) on Sundays.`
- **KDF answer** (✅): On Sunday, Admon was scheduled for the 8 am - 4 pm (Day Shift).
- **Mem0 answer** (❌): The specific rotation for Admon on a Sunday is not provided in the facts. The details of the shift rotation for each agent, including Admon, were not included.
- Mem0 retrieval recall: 0.000, haystack sessions: 1

### 8a2466db (single-session-preference, answer_type=long_explanation)

- **Question**: Can you recommend some resources where I can learn more about video editing?
- **Ground truth**: `The user would prefer responses that suggest resources specifically tailored to Adobe Premiere Pro, especially those that delve into its advanced settings. They might not prefer general video editing resources or resources related to other video editing software.`
- **KDF answer** (✅): I don't have specific resources listed, but you can explore online tutorials, Adobe's official documentation, and video editing forums to learn more about advanced settings in Adobe Premiere Pro and color grading techniques.
- **Mem0 answer** (❌): User is trying to learn more about advanced settings in Adobe Premiere Pro for video editing.
- Mem0 retrieval recall: 0.000, haystack sessions: 1

### 58ef2f1c (single-session-user, answer_type=date)

- **Question**: When did I volunteer at the local animal shelter's fundraising dinner?
- **Ground truth**: `February 14th`
- **KDF answer** (✅): You volunteered at the "Love is in the Air" fundraising dinner on Valentine's Day in February.
- **Mem0 answer** (❌): You volunteered at the 'Love is in the Air' fundraising dinner in February 2026.
- Mem0 retrieval recall: 0.000, haystack sessions: 1

### gpt4_2312f94c (temporal-reasoning, answer_type=short_fact)

- **Question**: Which device did I got first, the Samsung Galaxy S22 or the Dell XPS 13?
- **Ground truth**: `Samsung Galaxy S22`
- **KDF answer** (✅): You got the Samsung Galaxy S22 first on February 20th, followed by the Dell XPS 13, which arrived on February 25th.
- **Mem0 answer** (❌): You pre-ordered the Dell XPS 13 laptop on January 28, 2026, which arrived on February 25, 2026. You purchased the Samsung Galaxy S22 from Best Buy on February 20, 2026, making the Dell XPS 13 the first device you received.
- Mem0 retrieval recall: 0.000, haystack sessions: 2

## mem0_only (n=54)

### 830ce83f (knowledge-update, answer_type=short_fact)

- **Question**: Where did Rachel move to after her recent relocation?
- **Ground truth**: `the suburbs`
- **KDF answer** (❌): Rachel moved to a new apartment in Chicago.
- **Mem0 answer** (✅): Rachel recently moved back to the suburbs after living in Chicago.
- Mem0 retrieval recall: 0.000, haystack sessions: 2

### 0a995998 (multi-session, answer_type=number)

- **Question**: How many items of clothing do I need to pick up or return from a store?
- **Ground truth**: `3`
- **KDF answer** (❌): You need to pick up your dry cleaning for the navy blue blazer and return the boots to Zara that you exchanged for a larger size.
- **Mem0 answer** (✅): You need to pick up a new pair of boots from Zara and return the boots you exchanged for a larger size.
- Mem0 retrieval recall: 0.000, haystack sessions: 3

### 06878be2 (single-session-preference, answer_type=date)

- **Question**: Can you suggest some accessories that would complement my current photography setup?
- **Ground truth**: `The user would prefer suggestions of Sony-compatible accessories or high-quality photography gear that can enhance their photography experience. They may not prefer suggestions of other brands' equipment or low-quality gear.`
- **KDF answer** (❌): Consider getting a sturdy camera bag for protection, a lens cleaning kit for maintaining your lenses, and a tripod for stability during shoots. Additionally, a wireless remote trigger can enhance your shooting flexibility.
- **Mem0 answer** (✅): You might consider getting a comfortable and durable camera bag specifically designed for Sony cameras, as well as a case or pouch for your Godox V1 flash, such as the Godox V1 Hard Case or Think Tank Photo Flash Pouch. Additionally, an external battery pack like the Anker PowerCore 20000 PD could be useful for charging your Sony A7R IV on the go.
- Mem0 retrieval recall: 0.000, haystack sessions: 1

### bc8a6e93_abs (single-session-user, answer_type=long_explanation)

- **Question**: What did I bake for my uncle's birthday party?
- **Ground truth**: `You did not mention this information. You mentioned baking for your niece's birthday party but not your uncle's`
- **KDF answer** (❌): You baked a lemon cake or tart, as lemon curd is a fantastic choice for a dinner party dessert and pairs beautifully with various cakes and tarts.
- **Mem0 answer** (✅): You did not mention baking anything for your uncle's birthday party. You had success with a lemon blueberry cake for your niece's birthday party.
- Mem0 retrieval recall: 0.000, haystack sessions: 1

### gpt4_0b2f1d21 (temporal-reasoning, answer_type=short_fact)

- **Question**: Which event happened first, the purchase of the coffee maker or the malfunction of the stand mixer?
- **Ground truth**: `The malfunction of the stand mixer`
- **KDF answer** (❌): The purchase of the coffee maker happened first, as it was bought about three weeks ago, while the stand mixer malfunction occurred last month and took two weeks to repair.
- **Mem0 answer** (✅): The malfunction of the stand mixer happened first, as it was taken to the repair shop last month, while the user bought the coffee maker about three weeks ago.
- Mem0 retrieval recall: 0.000, haystack sessions: 2

### 852ce960 (knowledge-update, answer_type=short_fact)

- **Question**: What was the amount I was pre-approved for when I got my mortgage from Wells Fargo?
- **Ground truth**: `$400,000`
- **KDF answer** (❌): You were pre-approved for $350,000 from Wells Fargo.
- **Mem0 answer** (✅): You were pre-approved for a mortgage amount of $400,000 from Wells Fargo.
- Mem0 retrieval recall: 0.000, haystack sessions: 2

## both_wrong (n=98)

### 9ea5eabc (knowledge-update, answer_type=short_fact)

- **Question**: Where did I go on my most recent family trip?
- **Ground truth**: `Paris`
- **KDF answer** (❌): You went on your most recent family trip to Hawaii.
- **Mem0 answer** (❌): You recently went on a family trip to Hawaii, where you enjoyed snorkeling together.
- Mem0 retrieval recall: 0.000, haystack sessions: 2

### 3a704032 (multi-session, answer_type=number)

- **Question**: How many plants did I acquire in the last month?
- **Ground truth**: `3`
- **KDF answer** (❌): You acquired two plants in the last month: a peace lily and a succulent plant, which you bought two weeks ago. Additionally, you received a snake plant from your sister last month.
- **Mem0 answer** (❌): You acquired one snake plant from your sister last month.
- Mem0 retrieval recall: 0.000, haystack sessions: 3

### dc439ea3 (single-session-assistant, answer_type=short_fact)

- **Question**: I was looking back at our previous conversation about Native American powwows and I was wondering, which traditional game did you say was often performed by skilled dancers at powwows?
- **Ground truth**: `Hoop Dance`
- **KDF answer** (❌): The traditional game often performed by skilled dancers at powwows is the drum and dance competition, which is considered the highlight of the event.
- **Mem0 answer** (❌): The drum and dance competition is often performed by skilled dancers at powwows and is considered the highlight of the event.
- Mem0 retrieval recall: 0.000, haystack sessions: 1

### 0edc2aef (single-session-preference, answer_type=date)

- **Question**: Can you suggest a hotel for my upcoming trip to Miami?
- **Ground truth**: `The user would prefer suggestions of hotels in Miami that offer great views, possibly of the ocean or the city skyline, and have unique features such as a rooftop pool or a hot tub on the balcony. They may not prefer suggestions of basic or budget hotels without these features.`
- **KDF answer** (❌): I'm sorry, but I can only provide information about hotels in Seattle based on the facts provided. If you're looking for a hotel in Seattle, I can help with that!
- **Mem0 answer** (❌): The facts provided only pertain to a trip to Seattle and do not include any information about hotels in Miami. Therefore, I cannot suggest a hotel for your trip to Miami based on the given information.
- Mem0 retrieval recall: 0.000, haystack sessions: 1

### 08f4fc43 (temporal-reasoning, answer_type=short_fact)

- **Question**: How many days had passed between the Sunday mass at St. Mary's Church and the Ash Wednesday service at the cathedral?
- **Ground truth**: `30 days. 31 days (including the last day) is also acceptable.`
- **KDF answer** (❌): There were 30 days between the Sunday mass at St. Mary's Church on January 2nd and the Ash Wednesday service at the cathedral on February 1st.
- **Mem0 answer** (❌): There were 30 days between the Sunday mass at St. Mary's Church on January 2, 2026, and the Ash Wednesday service at the cathedral on February 1, 2026.
- Mem0 retrieval recall: 0.000, haystack sessions: 2

### 41698283 (knowledge-update, answer_type=short_fact)

- **Question**: What type of camera lens did I purchase most recently?
- **Ground truth**: `a 70-200mm zoom lens`
- **KDF answer** (❌): You recently purchased a new 50mm prime lens.
- **Mem0 answer** (❌): You recently got a new 50mm prime lens, which has been working out great for you.
- Mem0 retrieval recall: 0.000, haystack sessions: 2

