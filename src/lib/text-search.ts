import { match as matchPinyin } from "pinyin-pro";

const HAN_CHARACTERS = /[\u3400-\u9fff]/;
const ASCII_PINYIN_QUERY = /^[a-z]+$/i;

function matchesPinyinQuery(query: string, value: string) {
  if (!ASCII_PINYIN_QUERY.test(query) || !HAN_CHARACTERS.test(value)) {
    return false;
  }

  return matchPinyin(value, query.replace(/\s/gu, "")) !== null;
}

export function matchesTextQuery(query: string, ...values: string[]) {
  const normalized = query.trim().toLowerCase();
  const pinyinQuery = normalized.replace(/\s/gu, "");
  return (
    !normalized ||
    values.some(
      (value) =>
        value.toLowerCase().includes(normalized) ||
        matchesPinyinQuery(pinyinQuery, value),
    )
  );
}
