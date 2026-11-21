# Test strategy

All three candidates must produce one canonical block per equal fitted run.
The Rust tests cover empty input, backward pooling, equal levels, signed
extremes and rejection above the admitted input length. The Python runner
checks all returned blocks and expands them to exact rational fits. It compares
small inputs with direct consecutive-partition enumeration. It checks coverage,
mean order and every residual prefix on larger inputs, including the maximum
length and all ten timed shapes.

The oracle uses partition costs and exact fractions. It shares neither the
stack operations nor the cross-product comparator with the candidates.
The example `[6, 2, 3, 8]` exercises the second merge. A whole-array constant
fit passes the output-order requirement and fails the residual certificate.
The campaign verifies one independently computed digest per timed process.
