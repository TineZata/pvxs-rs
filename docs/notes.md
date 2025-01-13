https://indico.fhi-berlin.mpg.de/event/52/contributions/587/attachments/201/628/epics-pvxs-202010.pdf

Ex: Sync. Client GET
```
#include <iostream> 
#include <pvxs/client.h> 

using namespace pvxs; 

client::Context ctxt(client::Config::fromEnv().build()); 

Value result(ctxt.get(“pv:name”) .exec()->wait(5.0); // wait() throws on timeout 

std::cout<<result[“value”];
```

Ex: Async. Client GET 

```
#include <iostream> 
#include <pvxs/client.h> 

using namespace pvxs; 

auto ctxt(client::Config::fromEnv().build()); 
auto oper(ctxt.get(“pv:name”)
    .result([](Result&& result) {
        // on PVA worker thread
        std::cout<<result()[“value”];  // result() throws for remote error
    })
        .exec();
);
```

PV data structure
```
structure
  double value
  int severity
  string status
  structure timeStamp
    long secondsPastEpoch
    int nanoseconds
    int userTag
  structure display
    double limitLow
    double limitHigh
    double units
```

Run IOC on windows
```
..\..\bin\windows-x64\testPVA.exe st.cmd
```
