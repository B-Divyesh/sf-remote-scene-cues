import { describe, expect, it } from 'vitest';
import { csvEscape, parseCues } from './lib';

describe('cue sheet parsing', () => {
  it('parses scene and cue around the first separator', () => {
    expect(parseCues('Lobby | House open\nFinale | Lights | blackout')).toEqual([
      { scene: 'Lobby', name: 'House open' },
      { scene: 'Finale', name: 'Lights | blackout' }
    ]);
  });
  it('uses Scene for a plain cue line and ignores blanks', () => {
    expect(parseCues('\nStart music\n')).toEqual([{ scene: 'Scene', name: 'Start music' }]);
  });
  it('escapes CSV fields safely', () => expect(csvEscape('Cue "GO"')).toBe('"Cue ""GO"""'));
});
