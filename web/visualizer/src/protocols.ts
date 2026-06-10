export const P = 97;
export const mod = (n: number) => ((n % P) + P) % P;
export const add = (a: number, b: number) => mod(a + b);
export const mul = (a: number, b: number) => mod(a * b);
export const pow = (a: number, n: number) => { let r=1,b=mod(a); while(n){if(n&1)r=mul(r,b);b=mul(b,b);n>>=1;} return r; };
export const inv = (a: number) => { if(mod(a)===0) throw new Error('pole'); return pow(a,P-2); };
export type Round={g0:number;g1:number;challenge:number;claimIn:number;claimOut:number;remaining:number[]};
export function sumcheck(values:number[], claimed?:number){
  let layer=values.map(mod), claim=claimed===undefined?mod(layer.reduce((a,b)=>a+b,0)):mod(claimed); const initial=claim; const rounds:Round[]=[];
  for(let i=0;layer.length>1;i++){
    let g0=0,g1=0; for(let j=0;j<layer.length;j+=2){g0=add(g0,layer[j]);g1=add(g1,layer[j+1]);}
    const challenge=mod(claim*17+g0*31+g1*43+i*13+7); const next=add(mul(g0,1-challenge),mul(g1,challenge));
    layer=Array.from({length:layer.length/2},(_,j)=>add(mul(layer[j*2],1-challenge),mul(layer[j*2+1],challenge)));
    rounds.push({g0,g1,challenge,claimIn:claim,claimOut:next,remaining:layer}); claim=next;
  }
  return {claimed:initial,rounds,final:layer[0],accepted:rounds.every(r=>add(r.g0,r.g1)===r.claimIn)&&claim===layer[0]};
}

export function parseValues(input: string) {
  const values = input
    .split(/[\s,]+/)
    .filter(Boolean)
    .map((raw) => {
      const n = Number(raw);
      if (!Number.isSafeInteger(n)) {
        throw new Error("Enter only finite integer field elements.");
      }
      return mod(n);
    });

  if (!values.length || (values.length & (values.length - 1)) !== 0) {
    throw new Error("Enter 2, 4, 8, or 16 integers.");
  }

  return values;
}

export const fingerprint=(xs:number[],beta:number)=>xs.reduce((a,x)=>mul(a,add(beta,x)),1);
export const rational=(xs:number[],beta:number)=>xs.reduce((a,x)=>add(a,inv(add(beta,x))),0);
export function metrics(n:number){const levels=Math.log2(n); return {product:{passes:levels+1,peak:n,read:n*32*(levels+1),write:n*32*levels,ops:n-1},rational:{passes:1,peak:3,read:n*32,write:0,ops:n*2}};}

export type TranscriptJson = {
  version: 1;
  protocol: 'sumcheck';
  field: { modulus: number };
  claim: { num_variables: number; claimed_sum: number; oracle_evaluations: number[] };
  rounds: Array<{ round: number; g_at_zero: number; g_at_one: number; challenge: number }>;
  final: { point: number[]; oracle_evaluation: number };
};

/** Builds the versioned JSON envelope consumed by snark-lab-cli. */
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
