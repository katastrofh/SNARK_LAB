import { useMemo, useState } from 'react';
import {
  Activity,
  ArrowRight,
  Binary,
  BookOpen,
  Braces,
  Check,
  ChevronRight,
  CircleDot,
  Database,
  Download,
  Fingerprint,
  Gauge,
  Info,
  Layers3,
  LockKeyhole,
  Play,
  RotateCcw,
  ShieldCheck,
  Shuffle,
  Sigma,
  Sparkles,
  TriangleAlert,
  X,
  Zap,
} from 'lucide-react';
import {
  equalityWeights,
  fingerprint,
  metrics,
  mod,
  parseClaim,
  parseValues,
  P,
  rational,
  sumcheck,
  sumcheckTranscript,
} from './protocols';

type Tab = 'sumcheck' | 'zerocheck' | 'permcheck' | 'scribe';
type Icon = typeof Sigma;

const tabs: Array<{ id: Tab; label: string; eyebrow: string; icon: Icon }> = [
  { id: 'sumcheck', label: 'Sumcheck', eyebrow: 'Reduce a hypercube sum', icon: Sigma },
  { id: 'zerocheck', label: 'Zerocheck', eyebrow: 'Randomly mix constraints', icon: CircleDot },
  { id: 'permcheck', label: 'PermCheck', eyebrow: 'Compare tagged multisets', icon: Shuffle },
  { id: 'scribe', label: 'Streaming', eyebrow: 'Model prover I/O', icon: Database },
];

const formatBytes = (bytes: number) => {
  if (bytes >= 2 ** 30) return `${(bytes / 2 ** 30).toFixed(1)} GiB`;
  if (bytes >= 2 ** 20) return `${(bytes / 2 ** 20).toFixed(1)} MiB`;
  return `${(bytes / 2 ** 10).toFixed(1)} KiB`;
};

function downloadJson(name: string, value: unknown) {
  const blob = new Blob([`${JSON.stringify(value, null, 2)}\n`], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = name;
  anchor.rel = 'noopener';
  anchor.click();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}

function Formula({ children }: { children: React.ReactNode }) {
  return <code className="formula">{children}</code>;
}

function Status({ ok, children }: { ok: boolean; children: React.ReactNode }) {
  return (
    <span className={`status ${ok ? 'status-ok' : 'status-bad'}`}>
      {ok ? <Check size={14} /> : <X size={14} />}
      {children}
    </span>
  );
}

function Header({ active, onChange }: { active: Tab; onChange: (tab: Tab) => void }) {
  return (
    <header className="site-header">
      <button className="brand" onClick={() => onChange('sumcheck')} aria-label="Open Sumcheck lab">
        <span className="brand-mark"><Braces size={20} /></span>
        <span>snark<span>-lab</span></span>
      </button>
      <nav aria-label="Protocol labs">
        {tabs.map(({ id, label, icon: TabIcon }) => (
          <button key={id} className={active === id ? 'active' : ''} onClick={() => onChange(id)}>
            <TabIcon size={15} />{label}
          </button>
        ))}
      </nav>
      <div className="header-meta">
        <span className="core-pill"><ShieldCheck size={14} /> Rust core: BLS12-381 + Merlin</span>
        <span className="browser-pill">Browser: F<sub>97</sub></span>
      </div>
    </header>
  );
}

function SecurityBoundary() {
  return (
    <aside className="security-boundary" aria-label="Security boundary">
      <LockKeyhole size={18} />
      <div>
        <strong>Know which path you are using.</strong>
        <p>This page is an inspectable F₉₇ model. The Rust crates use large fields and Merlin Fiat–Shamir; commitment-backed oracle openings remain roadmap work.</p>
      </div>
      <span className="boundary-path">docs/sumcheck.md <ArrowRight size={14} /></span>
    </aside>
  );
}

function PageIntro({ tab, title, accent, children }: { tab: Tab; title: string; accent: string; children: React.ReactNode }) {
  const item = tabs.find(candidate => candidate.id === tab)!;
  return (
    <section className="page-intro">
      <div className="eyebrow"><Sparkles size={13} /> {item.eyebrow}</div>
      <h1>{title} <em>{accent}</em></h1>
      <p>{children}</p>
    </section>
  );
}

function SumcheckLab() {
  const [tableText, setTableText] = useState('3, 1, 4, 1, 5, 9, 2, 6');
  const [claimText, setClaimText] = useState('31');
  const [step, setStep] = useState(0);

  const parsed = useMemo(() => {
    try {
      return { values: parseValues(tableText), claim: parseClaim(claimText), error: '' };
    } catch (error) {
      return { values: [] as number[], claim: 0, error: (error as Error).message };
    }
  }, [tableText, claimText]);
  const proof = useMemo(
    () => parsed.values.length ? sumcheck(parsed.values, parsed.claim) : null,
    [parsed],
  );
  const rounds = proof?.rounds ?? [];
  const finalVisible = Boolean(proof && step > rounds.length);
  const progress = proof ? Math.min(100, (step / (rounds.length + 1)) * 100) : 0;

  const resetProgress = () => setStep(0);
  return (
    <main>
      <PageIntro tab="sumcheck" title="Turn an exponential sum into" accent="one point.">
        Follow every prover polynomial, consistency check, and challenge as one Boolean variable disappears per round.
      </PageIntro>
      <SecurityBoundary />

      <div className="lab-grid sumcheck-grid">
        <section className="card controls-card">
          <div className="card-heading">
            <span className="step-index">01</span>
            <div><h2>Define the statement</h2><p>Values are reduced into the browser field F₉₇.</p></div>
          </div>
          <label htmlFor="sumcheck-table">Evaluation table <small>POWER OF TWO</small></label>
          <textarea
            id="sumcheck-table"
            value={tableText}
            onChange={event => { setTableText(event.target.value); resetProgress(); }}
            spellCheck={false}
          />
          <div className="input-meta">
            <span><Layers3 size={14} /> {parsed.values.length} values</span>
            <span>{parsed.values.length ? Math.log2(parsed.values.length) : 0} variables</span>
          </div>
          <label htmlFor="sumcheck-claim">Claimed sum <small>MOD {P}</small></label>
          <input
            id="sumcheck-claim"
            inputMode="numeric"
            value={claimText}
            onChange={event => { setClaimText(event.target.value); resetProgress(); }}
          />
          {parsed.error && <p className="inline-error"><TriangleAlert size={14} />{parsed.error}</p>}
          <button
            className="primary-action"
            disabled={!proof}
            onClick={() => setStep(current => current > rounds.length ? 0 : current + 1)}
          >
            {step === 0 ? <><Play size={16} /> Start transcript</> : step <= rounds.length ? <><ChevronRight size={17} /> Reveal round {step}</> : <><RotateCcw size={16} /> Reset</>}
          </button>
          <button
            className="secondary-action"
            disabled={!proof}
            onClick={() => proof && downloadJson('sumcheck-transcript.json', sumcheckTranscript(parsed.values, parsed.claim))}
          >
            <Download size={16} /> Export educational JSON
          </button>
        </section>

        <section className="card transcript-card" aria-live="polite">
          <div className="transcript-header">
            <div><span className="section-label">02 / PROTOCOL TRACE</span><h2>Prover ↔ verifier</h2></div>
            {proof && <Status ok={finalVisible && proof.accepted}>{finalVisible ? (proof.accepted ? 'Accepted' : 'Rejected') : 'In progress'}</Status>}
          </div>
          <progress className="progress-track" max={100} value={progress} aria-label="Protocol progress" />

          {step === 0 ? (
            <div className="empty-state">
              <div className="protocol-orbit"><Sigma size={30} /><i /><i /><i /></div>
              <h3>Ready to collapse {parsed.values.length || 'the'} evaluations</h3>
              <p>Start the transcript to see why checking <Formula>gᵢ(0)+gᵢ(1)=Hᵢ₋₁</Formula> preserves the claim.</p>
            </div>
          ) : (
            <div className="round-list">
              <div className="initial-claim"><span>Initial claim</span><Formula>H₀ = {proof?.claimed}</Formula></div>
              {rounds.slice(0, step).map((round, index) => (
                <article className="round-card" key={index}>
                  <div className="round-marker">{index + 1}</div>
                  <div className="round-content">
                    <div className="round-title"><strong>Round {index + 1}</strong><span>{round.remaining.length} folded values remain</span></div>
                    <div className="protocol-message">
                      <span className="actor actor-prover">P</span>
                      <div><small>PROVER BINDS g{index + 1}</small><Formula>g(0)={round.g0} · g(1)={round.g1}</Formula></div>
                    </div>
                    <div className="consistency-check"><Check size={13} /> {round.g0} + {round.g1} ≡ {mod(round.g0 + round.g1)} = H{index}</div>
                    <div className="protocol-message verifier-message">
                      <span className="actor actor-verifier">V</span>
                      <div><small>CHALLENGE AFTER MESSAGE</small><Formula>r{index + 1}={round.challenge}</Formula></div>
                      <ArrowRight size={15} />
                      <Formula>H{index + 1}={round.claimOut}</Formula>
                    </div>
                    <div className="fold-strip">
                      {round.remaining.slice(0, 12).map((value, valueIndex) => <i key={valueIndex}>{value}</i>)}
                    </div>
                  </div>
                </article>
              ))}
              {finalVisible && proof && (
                <div className={`verdict ${proof.accepted ? 'verdict-ok' : 'verdict-bad'}`}>
                  <ShieldCheck size={26} />
                  <div><strong>{proof.accepted ? 'Final oracle check matches' : 'Claim rejected'}</strong><p>Folded claim {rounds.at(-1)?.claimOut} · oracle evaluation {proof.final}</p></div>
                </div>
              )}
            </div>
          )}
        </section>

        <aside className="card explainer-card">
          <span className="section-label">WHY THIS IS SOUND</span>
          <h3>A dishonest polynomial must survive a fresh point.</h3>
          <ol className="explanation-steps">
            <li><b>Bind.</b><span>The prover sends the next univariate polynomial.</span></li>
            <li><b>Check.</b><span>Its Boolean endpoints must equal the prior claim.</span></li>
            <li><b>Challenge.</b><span>The verifier samples only after the message is fixed.</span></li>
            <li><b>Reduce.</b><span>One variable is replaced by the challenge.</span></li>
          </ol>
          <div className="complexity-grid"><div><small>PROVER</small><strong>O(N)</strong></div><div><small>ROUNDS</small><strong>log₂N</strong></div></div>
          <div className="production-note"><LockKeyhole size={16} /><span>Rust uses Merlin; this browser trace is deterministic for teaching and JSON replay.</span></div>
        </aside>
      </div>
    </main>
  );
}

function ZerocheckLab() {
  const [violation, setViolation] = useState(false);
  const constraints = [0, 0, violation ? 9 : 0, 0, 0, 0, 0, 0];
  const mixingPoint = [5, 11, 19];
  const weights = equalityWeights(mixingPoint);
  const weighted = constraints.map((value, index) => mod(value * weights[index]));
  const mixedClaim = mod(weighted.reduce((sum, value) => sum + value, 0));

  return (
    <main>
      <PageIntro tab="zerocheck" title="Prove every constraint is" accent="zero.">
        See how one transcript-derived point mixes an entire Boolean constraint table into a single Sumcheck claim.
      </PageIntro>
      <SecurityBoundary />
      <div className="zerocheck-layout">
        <section className="card constraint-card">
          <div className="card-heading"><span className="step-index">01</span><div><h2>Constraint oracle</h2><p>Toggle one bad row to follow the failure.</p></div></div>
          <div className="cube-grid">
            {constraints.map((value, index) => <div key={index} className={value ? 'violated' : ''}><small>{index.toString(2).padStart(3, '0')}</small><strong>{value}</strong></div>)}
          </div>
          <button className={violation ? 'danger-action' : 'secondary-action'} onClick={() => setViolation(value => !value)}>
            {violation ? <><X size={16} /> Remove violation</> : <><TriangleAlert size={16} /> Inject violation at 010</>}
          </button>
        </section>
        <div className="flow-arrow"><ArrowRight /></div>
        <section className="card mixing-card">
          <span className="section-label">02 / RANDOM MIXING</span>
          <h2>Weight with eq(τ, x)</h2>
          <Formula>τ = ({mixingPoint.join(', ')})</Formula>
          <div className="weight-grid">{weights.map((weight, index) => <span key={index}>eq(τ,{index}) <b>{weight}</b></span>)}</div>
          <p><Info size={14} /> In Rust, the constraint oracle is bound before Merlin derives τ.</p>
        </section>
        <div className="flow-arrow"><ArrowRight /></div>
        <section className="card result-card">
          <span className="section-label">03 / SUMCHECK CLAIM</span>
          <div className={`result-orb ${mixedClaim === 0 ? 'result-ok' : 'result-bad'}`}>{mixedClaim === 0 ? <Check /> : <X />}</div>
          <h2>Σ eq(τ,x)f(x) = {mixedClaim}</h2>
          <Status ok={mixedClaim === 0}>{mixedClaim === 0 ? 'Zero claim holds' : 'Violation exposed'}</Status>
          <p>{mixedClaim === 0 ? 'Every weighted constraint contribution is zero.' : 'The bad row contributes a nonzero weighted term.'}</p>
        </section>
      </div>
      <section className="explanation-band"><BookOpen size={20} /><div><strong>The reduction</strong><p>If f vanishes everywhere, every weighted sum is zero. If not, a random equality weighting catches the nonzero polynomial except with field-bounded probability.</p></div><Formula>Σₓ eq(τ,x) · f(x) = 0</Formula></section>
    </main>
  );
}

function PermcheckLab() {
  const [mutated, setMutated] = useState(false);
  const left = [1, 5, 9, 2];
  const right = mutated ? [9, 2, 1, 6] : [9, 2, 1, 5];
  const beta = 11;
  const productLeft = fingerprint(left, beta);
  const productRight = fingerprint(right, beta);
  const rationalLeft = rational(left, beta);
  const rationalRight = rational(right, beta);

  return (
    <main>
      <PageIntro tab="permcheck" title="Compare multisets without" accent="sorting them.">
        Contrast a grand product with its logarithmic derivative and inspect why the rational form can stream with constant live state.
      </PageIntro>
      <SecurityBoundary />
      <div className="perm-layout">
        <section className="card column-card">
          <div className="card-heading"><span className="step-index">01</span><div><h2>Witness columns</h2><p>β = {beta} in this educational trace.</p></div></div>
          <Column label="A" values={left} tone="orange" />
          <Column label="B" values={right} tone="cyan" mutated={mutated} />
          <button className={mutated ? 'danger-action' : 'secondary-action'} onClick={() => setMutated(value => !value)}>
            {mutated ? <><RotateCcw size={16} /> Restore permutation</> : <><Shuffle size={16} /> Mutate last value</>}
          </button>
          <div className="production-note"><Fingerprint size={16} /><span>Rust binds tagged columns before deriving β and γ from Merlin.</span></div>
        </section>
        <div className="method-comparison">
          <MethodCard
            icon={Layers3}
            tone="orange"
            title="Grand product"
            formula="Πᵢ (β + aᵢ)"
            left={productLeft}
            right={productRight}
            detail="Natural algebraic identity, but product trees create intermediate layers and repeated memory traffic."
          />
          <MethodCard
            icon={Activity}
            tone="cyan"
            title="Rational stream"
            formula="Σᵢ 1 / (β + aᵢ)"
            left={rationalLeft}
            right={rationalRight}
            detail="One streaming accumulator; production code must define denominator-pole behavior and prove the relation."
          />
        </div>
      </div>
    </main>
  );
}

function Column({ label, values, tone, mutated }: { label: string; values: number[]; tone: string; mutated?: boolean }) {
  return <div className="column-row"><span>{label}</span><div>{values.map((value, index) => <i className={`${tone} ${mutated && index === values.length - 1 ? 'mutated' : ''}`} key={index}>{value}</i>)}</div></div>;
}

function MethodCard({ icon: MethodIcon, tone, title, formula, left, right, detail }: { icon: Icon; tone: string; title: string; formula: string; left: number; right: number; detail: string }) {
  const matches = left === right;
  return (
    <section className={`card method-card ${tone}`}>
      <div className="method-icon"><MethodIcon /></div>
      <span className="section-label">FINGERPRINT</span>
      <h2>{title}</h2>
      <Formula>{formula}</Formula>
      <div className="fingerprint-values"><b>{left}</b><span>vs</span><b>{right}</b></div>
      <Status ok={matches}>{matches ? 'Fingerprints match' : 'Mismatch detected'}</Status>
      <p>{detail}</p>
    </section>
  );
}

function StreamingLab() {
  const [power, setPower] = useState(20);
  const elements = 2 ** power;
  const estimate = metrics(elements);
  const productTraffic = estimate.product.read + estimate.product.write;
  const rationalTraffic = estimate.rational.read;
  const ratio = productTraffic / rationalTraffic;

  return (
    <main>
      <PageIntro tab="scribe" title="See where prover time becomes" accent="data movement.">
        Scale the witness and compare product-tree traffic against a single-pass rational accumulator.
      </PageIntro>
      <SecurityBoundary />
      <div className="stream-layout">
        <section className="card scale-card">
          <div className="card-heading"><span className="step-index">01</span><div><h2>Workload scale</h2><p>32-byte field-element model.</p></div></div>
          <div className="scale-number"><strong>2<sup>{power}</sup></strong><span>{elements.toLocaleString()} elements</span></div>
          <input aria-label="Workload exponent" type="range" min="10" max="26" value={power} onChange={event => setPower(Number(event.target.value))} />
          <div className="range-labels"><span>1K</span><span>1M</span><span>67M</span></div>
          <div className="ratio-callout"><Gauge size={20} /><div><strong>{ratio.toFixed(0)}× less modeled traffic</strong><p>for the rational stream at this scale</p></div></div>
        </section>
        <section className="card traffic-card">
          <div className="traffic-heading"><div><span className="section-label">02 / LOGICAL I/O</span><h2>Bytes moved through the prover</h2></div><span>MODEL, NOT HARDWARE COUNTERS</span></div>
          <TrafficBar label="Product tree reads" value={estimate.product.read} max={productTraffic} tone="orange" />
          <TrafficBar label="Product tree writes" value={estimate.product.write} max={productTraffic} tone="gold" />
          <TrafficBar label="Rational stream reads" value={rationalTraffic} max={productTraffic} tone="cyan" />
          <div className="metric-grid">
            <Metric label="PRODUCT PASSES" value={String(estimate.product.passes)} />
            <Metric label="PRODUCT PEAK" value={formatBytes(estimate.product.peak * 32)} />
            <Metric label="STREAM PEAK" value="96 B" />
            <Metric label="TRAFFIC SAVED" value={formatBytes(productTraffic - rationalTraffic)} />
          </div>
        </section>
      </div>
      <section className="pipeline-card">
        <div><Binary size={20} /><span>Witness stream</span></div><ArrowRight />
        <div className="tree-visual"><i /><i /><i /><i /><i /><i /><span>product layers</span></div><span className="versus">versus</span>
        <div className="accumulator-visual"><Zap size={18} /><Formula>acc += 1 / term</Formula><span>constant live state</span></div>
      </section>
    </main>
  );
}

function TrafficBar({ label, value, max, tone }: { label: string; value: number; max: number; tone: string }) {
  return <div className="traffic-row"><div><span>{label}</span><b>{formatBytes(value)}</b></div><progress className={`bar-track ${tone}`} max={max} value={value} aria-label={`${label}: ${formatBytes(value)}`} /></div>;
}

function Metric({ label, value }: { label: string; value: string }) {
  return <div><small>{label}</small><strong>{value}</strong></div>;
}

export default function App() {
  const [tab, setTab] = useState<Tab>('sumcheck');
  return (
    <div className="app-shell">
      <Header active={tab} onChange={setTab} />
      {tab === 'sumcheck' && <SumcheckLab />}
      {tab === 'zerocheck' && <ZerocheckLab />}
      {tab === 'permcheck' && <PermcheckLab />}
      {tab === 'scribe' && <StreamingLab />}
      <footer><span><ShieldCheck size={14} /> No telemetry · local computation</span><p>Educational browser model + production-oriented Rust protocol core.</p><code>docs/</code></footer>
    </div>
  );
}
