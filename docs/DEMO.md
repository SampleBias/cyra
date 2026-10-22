# Initial offline terminal demonstration

Delivery target: Sprint 06 / M3. **This is an executable acceptance script specification; neither the binary nor the fixture files exist yet.** Sprint 02 materializes profile assets, Sprint 04 supplies the toy SCM, and Sprints 05–06 supply dynamics and complete replay.

## Scope and fixture design

Use one illustrative WJ-MSC profile bundle with two explicitly separate components:

- A time-dependent licensing/secretion mechanism using illustrative biological parameters.
- A dimensionless toy SCM that demonstrates the three causal levels using named licensing/secretion proxies. It is a mathematical component of the demo bundle, not a fitted physiological link to the ODE.

The first counterfactual is supported for the toy SCM. Do not claim it reconstructs an individual cell's unobserved physiological history or performs nonlinear ODE abduction. Adding dynamic biological counterfactuals requires a later supported observation/abduction model and its own tests.

The toy model is `C = Uc`, `X = C + Ux`, `Y = 2X + 3C + Uy`, with independent standard-normal exogenous variables. `C`, `X`, and `Y` are dimensionless background, licensing and secretion proxies; coefficients are illustrative mathematical constants. Exact synthetic evidence `C=1, X=2, Y=8` implies `Uc=1, Ux=1, Uy=1`. The counterfactual `Y` under `do(X=0)` is therefore `4`.

For this model, `E[Y | X=x]=3.5x` but `E[Y | do(X=x)]=2x`. Under `do(X=2)`, the population mean is `4`. With only evidence `X=2`, `Y_0` has mean `3` and variance `5.5`. These are distinct questions and must receive distinct labels and uncertainty fields. All facts here follow from the constructed equations, not from experimental MSC observations.

For the dynamic fixture, choose `dL/dt = a(1-L)-bL`, with `a=1 h^-1`, `b=0.5 h^-1`, and `L(0)=0`. The analytic licensing solution is `L(t)=(2/3)(1-exp(-1.5t))`, with `t` in hours. Downstream secretion uses an explicitly unit-checked production/clearance balance. All profile parameters are illustrative; the time axis does not make them empirically validated.

## Proposed single-terminal sequence

Prerequisites when implemented: a Rust-built `vmcell` binary and its bundled demo assets; no network, credentials, Python or JavaScript. Run in a new directory because artifact overwrites are prohibited. The `--demo-assets` option creates the named requests/scenarios below; the script must not rely on manually authored hidden inputs.

```sh
vmcell --offline init --profile wj-msc-illustrative --demo-assets --out ./cyra-demo
vmcell --offline graph inspect --model ./cyra-demo/toy-scm.json
vmcell --offline graph validate --model ./cyra-demo/toy-scm.json
vmcell --offline graph inspect --model ./cyra-demo/licensing-transition-graph.json
vmcell --offline graph validate --model ./cyra-demo/licensing-transition-graph.json
vmcell --offline observe --profile ./cyra-demo/profile.json
vmcell --offline simulate --profile ./cyra-demo/profile.json --scenario ./cyra-demo/baseline.json --seed 42 --out ./cyra-demo/runs/baseline
vmcell --offline simulate --profile ./cyra-demo/profile.json --scenario ./cyra-demo/licensing-pulse.json --seed 42 --out ./cyra-demo/runs/pulse
vmcell --offline intervene --model ./cyra-demo/toy-scm.json --request ./cyra-demo/do-x-2.json --seed 42 --out ./cyra-demo/runs/intervention
vmcell --offline counterfactual --model ./cyra-demo/toy-scm.json --request ./cyra-demo/cf-x-0.json --seed 42 --out ./cyra-demo/runs/counterfactual
vmcell --offline explain --result ./cyra-demo/runs/counterfactual/result.json
vmcell --offline provenance --artifact ./cyra-demo/runs/counterfactual/result.json
vmcell --offline reproduce --manifest ./cyra-demo/runs/baseline/manifest.json --out ./cyra-demo/replays/baseline
vmcell --offline reproduce --manifest ./cyra-demo/runs/counterfactual/manifest.json --out ./cyra-demo/replays/counterfactual
```

`do-x-2.json` requests the specified-SCM population mean under `do(X=2)`. `cf-x-0.json` requests the outcome under `do(X=0)` conditioned on the exact synthetic evidence above. Both explicitly select `basis=specified_scm`. Baseline and pulse scenarios record all state values, times, parameters, intervention event order and solver settings. Each run directory includes input/result manifests, scientific outputs and a separate execution record.

## Acceptance evidence

1. Run all commands successfully in a network-disabled test environment with no AI credentials.
2. Graph inspection shows the actual equations and provenance; profile inspection labels every synthetic/illustrative field.
3. Licensing state agrees with its analytic trajectory within declared solver tolerance; pulse events occur exactly at configured boundaries.
4. The intervention mean is `4`; the counterfactual is `4` for a different mathematical reason. The explanation must show the difference and the abduced residuals, rather than relying on their coincident numeric values.
5. A separate association query at `X=2` returns conditional mean `7`; do not confuse this with the individual synthetic evidence `Y=8`.
6. A partial-evidence counterfactual returns the mean/variance above, not an arbitrary point background.
7. Reproduction verifies hashes and reproduces scientific results. Execution timestamps and durations may differ and must be excluded from scientific-output equivalence.
8. Tampering with an input or parameter fails replay integrity before computation. An unsupported nonlinear counterfactual fails with exit 3 and no numeric outcome.
9. Repeat the sequence through JSONL using a Rust integration harness and schema-validate every line; capture stdout/stderr separately.

Store the terminal transcript, JSONL transcript, analytic comparisons, model/fixture hashes and replay report in Sprint 06 evidence. A successful demonstration proves the implemented mathematics and workflow on illustrative fixtures; it is not empirical validation of MSC physiological prediction.
