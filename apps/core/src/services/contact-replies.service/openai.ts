import OpenAI from 'openai';

let openai: OpenAI | undefined;

export function getOpenAI() {
  openai ??= new OpenAI();
  return openai;
}

