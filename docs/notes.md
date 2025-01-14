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

Example structure from 
```
rec:X from 172.23.238.95:5075
struct "epics:nt/NTScalar:1.0" {
    double value
    struct "alarm_t" {
        int32_t severity
        int32_t status
        string message
    } alarm
    struct {
        int64_t secondsPastEpoch
        int32_t nanoseconds
        int32_t userTag
    } timeStamp
    struct {
        double limitLow
        double limitHigh
        string description
        string units
        int32_t precision
        struct "enum_t" {
            int32_t index
            string[] choices
        } form
    } display
    struct "control_t" {
        double limitLow
        double limitHigh
        double minStep
    } control
    struct "valueAlarm_t" {
        bool active
        double lowAlarmLimit
        double lowWarningLimit
        double highWarningLimit
        double highAlarmLimit
        int32_t lowAlarmSeverity
        int32_t lowWarningSeverity
        int32_t highWarningSeverity
        int32_t highAlarmSeverity
        int8_t hysteresis
    } valueAlarm
}
```

 bindgen .\client.h -o clientBindgen.rs -- -x c++ -std=c++17 -I../../include -I../../../epics-base/include -I../../../epics-base/include/compiler/msvc -I../../../epics-base/include/os/WIN32
