pragma circom 2.0.0;

template Add() {
    signal input a;
    signal input b;
    signal output c;

    c <== a + b;
}

template ContinuousCompute(n) {
    signal input initial_state;
    signal input steps[n];
    signal output final_state;

    component adders[n];

    adders[0] = Add();
    adders[0].a <== initial_state;
    adders[0].b <== steps[0];

    for (var i = 1; i < n; i++) {
        adders[i] = Add();
        adders[i].a <== adders[i-1].c;
        adders[i].b <== steps[i];
    }

    final_state <== adders[n-1].c;
}

component main {public [initial_state,steps]}= ContinuousCompute(4);

