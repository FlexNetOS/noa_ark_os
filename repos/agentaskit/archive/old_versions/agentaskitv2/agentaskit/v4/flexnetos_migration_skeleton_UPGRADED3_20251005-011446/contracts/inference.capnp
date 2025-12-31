
@0xa1a1a1a1a1a1a1a1;
using Cxx = import "/capnp/c++.capnp";
$Cxx.namespace("flex::inference");

struct InferenceRequest { input @0 :Data; traceId @1 :Text; }
struct InferenceReply   { output @0 :Data; modelHash @1 :Text; }

interface Inference {
  predict @0 (req :InferenceRequest) -> (rep :InferenceReply);
}
