import { describe, expect, it } from 'vitest';
import { readableDescription } from './display';

describe('readableDescription', () => {
  it('drops HTML, badge links and a tag cut off mid-way', () => {
    expect(
      readableDescription(
        '<p align="center"> <img src="./header.png" width="1000"> </p> Ollama GUI A web interface <img src="x'
      )
    ).toBe('Ollama GUI A web interface');
    expect(
      readableDescription('CICD https://github.com/a/b/badge.svg?branch=main Spring Boot app')
    ).toBe('CICD Spring Boot app');
  });

  it('leaves plain text and empty input alone', () => {
    expect(readableDescription('Marketing site for Aipairs')).toBe('Marketing site for Aipairs');
    expect(readableDescription(undefined)).toBe('');
  });
});
