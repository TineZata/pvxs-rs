# include "pvxs_wrapper.h"
#include <pvxs/version.h>

extern "C"{
    const char* pva_version_str()
    {
        return pvxs::version_str();
    }
}
