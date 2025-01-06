/* Generated file epicsVersion.h */

#ifndef INC_epicsVersion_H
#define INC_epicsVersion_H

#define EPICS_VERSION        7
#define EPICS_REVISION       0
#define EPICS_MODIFICATION   8
#define EPICS_PATCH_LEVEL    2
#define EPICS_DEV_SNAPSHOT   "-DEV"
#define EPICS_SITE_VERSION   ""

#define EPICS_VERSION_SHORT  "7.0.8.2"
#define EPICS_VERSION_FULL   "7.0.8.2-DEV"
#define EPICS_VERSION_STRING "EPICS 7.0.8.2-DEV"
#define epicsReleaseVersion  "EPICS R7.0.8.2-DEV"

#ifndef VERSION_INT
#  define VERSION_INT(V,R,M,P) ( ((V)<<24) | ((R)<<16) | ((M)<<8) | (P))
#endif
#define EPICS_VERSION_INT VERSION_INT(7, 0, 8, 2)

#endif /* INC_epicsVersion_H */
