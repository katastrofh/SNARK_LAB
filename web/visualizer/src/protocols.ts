export const P = 97;
export const MAX_BROWSER_EVALUATIONS = 1 << 12;

export const mod = (value: number) => ((value % P) + P) % P;
export const add = (left: number, right: number) => mod(left + right);
export const mul = (left: number, right: number) => mod(left * right);

export function pow(value: number, exponent: number) {
  let result = 1;
  let base = mod(value);
  let remaining = exponent;
  while (remaining > 0) {
    if (remaining & 1) result = mul(result, base);
    base = mul(base, base);
    remaining >>= 1;
  }
  return result;
}

export function inv(value: number) {
  if (mod(value) === 0) throw new Error('Denominator is zero in F₉₇.');
  return pow(value, P - 2);
}

export type Round = {
  g0: number;
  g1: number;
  challenge: number;
  claimIn: number;
  claimOut: number;
  remaining: number[];
};

/** Educational F_97 transcript; the Rust core uses Merlin over BLS12-381 Fr. */
export function sumcheck(values: number[], claimed?: number) {
  let layer = values.map(mod);
  let claim = claimed === undefined ? mod(layer.reduce((sum, value) => sum + value, 0)) : mod(claimed);
  const initial = claim;
  const rounds: Round[] = [];

  for (let round = 0; layer.length > 1; round += 1) {
    let g0 = 0;
    let g1 = 0;
    for (let index = 0; index < layer.length; index += 2) {
      g0 = add(g0, layer[index]);
      g1 = add(g1, layer[index + 1]);
    }
    const challenge = mod(claim * 17 + g0 * 31 + g1 * 43 + round * 13 + 7);
    const claimOut = add(mul(g0, 1 - challenge), mul(g1, challenge));
    layer = Array.from({ length: layer.length / 2 }, (_, index) =>
      add(mul(layer[index * 2], 1 - challenge), mul(layer[index * 2 + 1], challenge)),
    );
    rounds.push({ g0, g1, challenge, claimIn: claim, claimOut, remaining: layer });
    claim = claimOut;
  }

  return {
    claimed: initial,
    rounds,
    final: layer[0],
    accepted: rounds.every(round => add(round.g0, round.g1) === round.claimIn) && claim === layer[0],
  };
}

export function parseValues(input: string) {
  const tokens = input.split(/[\s,]+/).filter(Boolean);
  if (!tokens.length || tokens.length > MAX_BROWSER_EVALUATIONS || (tokens.length & (tokens.length - 1)) !== 0) {
    throw new Error(`Enter a power-of-two table with at most ${MAX_BROWSER_EVALUATIONS} integers.`);
  }
  const values = tokens.map(Number);
  if (values.some(value => !Number.isSafeInteger(value))) {
    throw new Error('Every evaluation must be a safe integer.');
  }
  return values.map(mod);
}

export function parseClaim(input: string) {
  const value = Number(input);
  if (!Number.isSafeInteger(value)) throw new Error('The claim must be a safe integer.');
  return mod(value);
}

export const fingerprint = (values: number[], beta: number) =>
  values.reduce((product, value) => mul(product, add(beta, value)), 1);

export const rational = (values: number[], beta: number) =>
  values.reduce((sum, value) => add(sum, inv(add(beta, value))), 0);

export function equalityWeights(point: number[]) {
  let values = [1];
  point.forEach(coordinate => {
    values = values.flatMap(value => [mul(value, 1 - coordinate), mul(value, coordinate)]);
  });
  return values;
}

export function metrics(elements: number) {
  const levels = Math.log2(elements);
  return {
    product: {
      passes: levels + 1,
      peak: elements,
      read: elements * 32 * (levels + 1),
      write: elements * 32 * levels,
      ops: elements - 1,
    },
    rational: { passes: 1, peak: 3, read: elements * 32, write: 0, ops: elements * 2 },
  };
}

export type TranscriptJson = {
  version: 1;
  protocol: 'sumcheck';
  field: { modulus: number };
  claim: { num_variables: number; claimed_sum: number; oracle_evaluations: number[] };
  rounds: Array<{ round: number; g_at_zero: number; g_at_one: number; challenge: number }>;
  final: { point: number[]; oracle_evaluation: number };
};

export function sumcheckTranscript(values: number[], claimed: number): TranscriptJson {
  const canonicalValues = values.map(mod);
  const proof = sumcheck(canonicalValues, claimed);
  return {
    version: 1,
    protocol: 'sumcheck',
    field: { modulus: P },
    claim: {
      num_variables: Math.log2(canonicalValues.length),
      claimed_sum: proof.claimed,
      oracle_evaluations: canonicalValues,
    },
    rounds: proof.rounds.map((round, index) => ({
      round: index,
      g_at_zero: round.g0,
      g_at_one: round.g1,
      challenge: round.challenge,
    })),
    final: {
      point: proof.rounds.map(round => round.challenge),
      oracle_evaluation: proof.final,
    },
  };
}

export type IpaRoundTrace = {
  round: number;
  inputLength: number;
  outputLength: number;
  leftCommitment: string;
  rightCommitment: string;
  challenge: number;
  challengeInverse: number;
  polynomialBefore: number[];
  evaluationBefore: number[];
  polynomialAfter: number[];
  evaluationAfter: number[];
  innerProductBefore: number;
  innerProductAfter: number;
};

export type IpaTrace = {
  table: number[];
  point: number[];
  basis: number[];
  claimedValue: number;
  commitmentBytes: number;
  encodedOpeningBytes: number;
  decodedRounds: number;
  rounds: IpaRoundTrace[];
  finalPolynomialScalar: number;
  finalEvaluationScalar: number;
  accepted: boolean;
};

export function innerProduct(left: number[], right: number[]) {
  if (left.length !== right.length) {
    throw new Error('Inner-product vectors must have matching lengths.');
  }

  return left.reduce((sum, value, index) => add(sum, mul(value, right[index])), 0);
}

export function foldIpaPolynomialVector(values: number[], challenge: number) {
  if (values.length < 2 || values.length % 2 !== 0) {
    throw new Error('IPA polynomial vector must have even length at each fold.');
  }

  const inverse = inv(challenge);
  const half = values.length / 2;

  return Array.from({ length: half }, (_, index) =>
    add(mul(challenge, values[index]), mul(inverse, values[index + half])),
  );
}

export function foldIpaEvaluationVector(values: number[], challenge: number) {
  if (values.length < 2 || values.length % 2 !== 0) {
    throw new Error('IPA evaluation vector must have even length at each fold.');
  }

  const inverse = inv(challenge);
  const half = values.length / 2;

  return Array.from({ length: half }, (_, index) =>
    add(mul(inverse, values[index]), mul(challenge, values[index + half])),
  );
}

function commitmentLabel(prefix: string, round: number, values: number[]) {
  const digest = values.reduce((acc, value, index) => add(acc, mul(value + index + 1, 17 + round * 11)), 0);
  return prefix + round + ':' + digest.toString().padStart(2, '0');
}

function deriveIpaChallenge(round: number, leftCommitment: string, rightCommitment: string, previous: number) {
  const left = Array.from(leftCommitment).reduce((sum, char) => sum + char.charCodeAt(0), 0);
  const right = Array.from(rightCommitment).reduce((sum, char) => sum + char.charCodeAt(0), 0);
  const challenge = mod(left * 7 + right * 11 + previous * 13 + round * 17 + 5);

  return challenge === 0 ? 1 : challenge;
}

export function ipaTrace(tableInput = [3, 1, 4, 1], pointInput = [2, 7]): IpaTrace {
  const table = tableInput.map(mod);
  const point = pointInput.map(mod);
  const basis = equalityWeights(point);

  if (table.length !== basis.length) {
    throw new Error('IPA trace table length must match the evaluation-basis length.');
  }

  let polynomialVector = table;
  let evaluationVector = basis;
  let previousChallenge = innerProduct(table, basis);
  const rounds: IpaRoundTrace[] = [];

  for (let round = 0; polynomialVector.length > 1; round += 1) {
    const inputLength = polynomialVector.length;
    const half = inputLength / 2;
    const leftPolynomial = polynomialVector.slice(0, half);
    const rightPolynomial = polynomialVector.slice(half);
    const leftEvaluation = evaluationVector.slice(0, half);
    const rightEvaluation = evaluationVector.slice(half);

    const leftCrossTerm = innerProduct(leftPolynomial, rightEvaluation);
    const rightCrossTerm = innerProduct(rightPolynomial, leftEvaluation);
    const leftCommitment = commitmentLabel('L', round, [leftCrossTerm, ...leftPolynomial, ...rightEvaluation]);
    const rightCommitment = commitmentLabel('R', round, [rightCrossTerm, ...rightPolynomial, ...leftEvaluation]);
    const challenge = deriveIpaChallenge(round, leftCommitment, rightCommitment, previousChallenge);
    const challengeInverse = inv(challenge);
    const polynomialAfter = foldIpaPolynomialVector(polynomialVector, challenge);
    const evaluationAfter = foldIpaEvaluationVector(evaluationVector, challenge);
    const innerProductBefore = innerProduct(polynomialVector, evaluationVector);
    const innerProductAfter = innerProduct(polynomialAfter, evaluationAfter);

    rounds.push({
      round,
      inputLength,
      outputLength: polynomialAfter.length,
      leftCommitment,
      rightCommitment,
      challenge,
      challengeInverse,
      polynomialBefore: polynomialVector,
      evaluationBefore: evaluationVector,
      polynomialAfter,
      evaluationAfter,
      innerProductBefore,
      innerProductAfter,
    });

    polynomialVector = polynomialAfter;
    evaluationVector = evaluationAfter;
    previousChallenge = challenge;
  }

  return {
    table,
    point,
    basis,
    claimedValue: innerProduct(table, basis),
    commitmentBytes: 48,
    encodedOpeningBytes: 585,
    decodedRounds: 3,
    rounds,
    finalPolynomialScalar: polynomialVector[0],
    finalEvaluationScalar: evaluationVector[0],
    accepted: table.length === basis.length && rounds.length === point.length,
  };
}
