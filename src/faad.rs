/* automatically generated by rust-bindgen */

pub const FAAD2_VERSION: &'static [u8; 4usize] = b"2.7\x00";
pub const MAIN: ::std::os::raw::c_uint = 1;
pub const LC: ::std::os::raw::c_uint = 2;
pub const SSR: ::std::os::raw::c_uint = 3;
pub const LTP: ::std::os::raw::c_uint = 4;
pub const HE_AAC: ::std::os::raw::c_uint = 5;
pub const ER_LC: ::std::os::raw::c_uint = 17;
pub const ER_LTP: ::std::os::raw::c_uint = 19;
pub const LD: ::std::os::raw::c_uint = 23;
pub const DRM_ER_LC: ::std::os::raw::c_uint = 27;
pub const RAW: ::std::os::raw::c_uint = 0;
pub const ADIF: ::std::os::raw::c_uint = 1;
pub const ADTS: ::std::os::raw::c_uint = 2;
pub const LATM: ::std::os::raw::c_uint = 3;
pub const NO_SBR: ::std::os::raw::c_uint = 0;
pub const SBR_UPSAMPLED: ::std::os::raw::c_uint = 1;
pub const SBR_DOWNSAMPLED: ::std::os::raw::c_uint = 2;
pub const NO_SBR_UPSAMPLED: ::std::os::raw::c_uint = 3;
pub const FAAD_FMT_16BIT: ::std::os::raw::c_uint = 1;
pub const FAAD_FMT_24BIT: ::std::os::raw::c_uint = 2;
pub const FAAD_FMT_32BIT: ::std::os::raw::c_uint = 3;
pub const FAAD_FMT_FLOAT: ::std::os::raw::c_uint = 4;
pub const FAAD_FMT_FIXED: ::std::os::raw::c_uint = 4;
pub const FAAD_FMT_DOUBLE: ::std::os::raw::c_uint = 5;
pub const LC_DEC_CAP: ::std::os::raw::c_uint = 1;
pub const MAIN_DEC_CAP: ::std::os::raw::c_uint = 2;
pub const LTP_DEC_CAP: ::std::os::raw::c_uint = 4;
pub const LD_DEC_CAP: ::std::os::raw::c_uint = 8;
pub const ERROR_RESILIENCE_CAP: ::std::os::raw::c_uint = 16;
pub const FIXED_POINT_CAP: ::std::os::raw::c_uint = 32;
pub const FRONT_CHANNEL_CENTER: ::std::os::raw::c_uint = 1;
pub const FRONT_CHANNEL_LEFT: ::std::os::raw::c_uint = 2;
pub const FRONT_CHANNEL_RIGHT: ::std::os::raw::c_uint = 3;
pub const SIDE_CHANNEL_LEFT: ::std::os::raw::c_uint = 4;
pub const SIDE_CHANNEL_RIGHT: ::std::os::raw::c_uint = 5;
pub const BACK_CHANNEL_LEFT: ::std::os::raw::c_uint = 6;
pub const BACK_CHANNEL_RIGHT: ::std::os::raw::c_uint = 7;
pub const BACK_CHANNEL_CENTER: ::std::os::raw::c_uint = 8;
pub const LFE_CHANNEL: ::std::os::raw::c_uint = 9;
pub const UNKNOWN_CHANNEL: ::std::os::raw::c_uint = 0;
pub const DRMCH_MONO: ::std::os::raw::c_uint = 1;
pub const DRMCH_STEREO: ::std::os::raw::c_uint = 2;
pub const DRMCH_SBR_MONO: ::std::os::raw::c_uint = 3;
pub const DRMCH_SBR_STEREO: ::std::os::raw::c_uint = 4;
pub const DRMCH_SBR_PS_STEREO: ::std::os::raw::c_uint = 5;
pub const FAAD_MIN_STREAMSIZE: ::std::os::raw::c_uint = 768;
pub type NeAACDecHandle = *mut ::std::os::raw::c_void;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct mp4AudioSpecificConfig {
    pub objectTypeIndex: ::std::os::raw::c_uchar,
    pub samplingFrequencyIndex: ::std::os::raw::c_uchar,
    pub samplingFrequency: ::std::os::raw::c_ulong,
    pub channelsConfiguration: ::std::os::raw::c_uchar,
    pub frameLengthFlag: ::std::os::raw::c_uchar,
    pub dependsOnCoreCoder: ::std::os::raw::c_uchar,
    pub coreCoderDelay: ::std::os::raw::c_ushort,
    pub extensionFlag: ::std::os::raw::c_uchar,
    pub aacSectionDataResilienceFlag: ::std::os::raw::c_uchar,
    pub aacScalefactorDataResilienceFlag: ::std::os::raw::c_uchar,
    pub aacSpectralDataResilienceFlag: ::std::os::raw::c_uchar,
    pub epConfig: ::std::os::raw::c_uchar,
    pub sbr_present_flag: ::std::os::raw::c_char,
    pub forceUpSampling: ::std::os::raw::c_char,
    pub downSampledSBR: ::std::os::raw::c_char,
}
#[test]
fn bindgen_test_layout_mp4AudioSpecificConfig() {
    assert_eq!(::std::mem::size_of::<mp4AudioSpecificConfig>() , 32usize ,
               concat ! ( "Size of: " , stringify ! ( mp4AudioSpecificConfig )
               ));
    assert_eq! (::std::mem::align_of::<mp4AudioSpecificConfig>() , 8usize ,
                concat ! (
                "Alignment of " , stringify ! ( mp4AudioSpecificConfig ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const mp4AudioSpecificConfig ) ) .
                objectTypeIndex as * const _ as usize } , 0usize , concat ! (
                "Alignment of field: " , stringify ! ( mp4AudioSpecificConfig
                ) , "::" , stringify ! ( objectTypeIndex ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const mp4AudioSpecificConfig ) ) .
                samplingFrequencyIndex as * const _ as usize } , 1usize ,
                concat ! (
                "Alignment of field: " , stringify ! ( mp4AudioSpecificConfig
                ) , "::" , stringify ! ( samplingFrequencyIndex ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const mp4AudioSpecificConfig ) ) .
                samplingFrequency as * const _ as usize } , 8usize , concat !
                (
                "Alignment of field: " , stringify ! ( mp4AudioSpecificConfig
                ) , "::" , stringify ! ( samplingFrequency ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const mp4AudioSpecificConfig ) ) .
                channelsConfiguration as * const _ as usize } , 16usize ,
                concat ! (
                "Alignment of field: " , stringify ! ( mp4AudioSpecificConfig
                ) , "::" , stringify ! ( channelsConfiguration ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const mp4AudioSpecificConfig ) ) .
                frameLengthFlag as * const _ as usize } , 17usize , concat ! (
                "Alignment of field: " , stringify ! ( mp4AudioSpecificConfig
                ) , "::" , stringify ! ( frameLengthFlag ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const mp4AudioSpecificConfig ) ) .
                dependsOnCoreCoder as * const _ as usize } , 18usize , concat
                ! (
                "Alignment of field: " , stringify ! ( mp4AudioSpecificConfig
                ) , "::" , stringify ! ( dependsOnCoreCoder ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const mp4AudioSpecificConfig ) ) .
                coreCoderDelay as * const _ as usize } , 20usize , concat ! (
                "Alignment of field: " , stringify ! ( mp4AudioSpecificConfig
                ) , "::" , stringify ! ( coreCoderDelay ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const mp4AudioSpecificConfig ) ) .
                extensionFlag as * const _ as usize } , 22usize , concat ! (
                "Alignment of field: " , stringify ! ( mp4AudioSpecificConfig
                ) , "::" , stringify ! ( extensionFlag ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const mp4AudioSpecificConfig ) ) .
                aacSectionDataResilienceFlag as * const _ as usize } , 23usize
                , concat ! (
                "Alignment of field: " , stringify ! ( mp4AudioSpecificConfig
                ) , "::" , stringify ! ( aacSectionDataResilienceFlag ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const mp4AudioSpecificConfig ) ) .
                aacScalefactorDataResilienceFlag as * const _ as usize } ,
                24usize , concat ! (
                "Alignment of field: " , stringify ! ( mp4AudioSpecificConfig
                ) , "::" , stringify ! ( aacScalefactorDataResilienceFlag )
                ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const mp4AudioSpecificConfig ) ) .
                aacSpectralDataResilienceFlag as * const _ as usize } ,
                25usize , concat ! (
                "Alignment of field: " , stringify ! ( mp4AudioSpecificConfig
                ) , "::" , stringify ! ( aacSpectralDataResilienceFlag ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const mp4AudioSpecificConfig ) ) . epConfig as
                * const _ as usize } , 26usize , concat ! (
                "Alignment of field: " , stringify ! ( mp4AudioSpecificConfig
                ) , "::" , stringify ! ( epConfig ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const mp4AudioSpecificConfig ) ) .
                sbr_present_flag as * const _ as usize } , 27usize , concat !
                (
                "Alignment of field: " , stringify ! ( mp4AudioSpecificConfig
                ) , "::" , stringify ! ( sbr_present_flag ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const mp4AudioSpecificConfig ) ) .
                forceUpSampling as * const _ as usize } , 28usize , concat ! (
                "Alignment of field: " , stringify ! ( mp4AudioSpecificConfig
                ) , "::" , stringify ! ( forceUpSampling ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const mp4AudioSpecificConfig ) ) .
                downSampledSBR as * const _ as usize } , 29usize , concat ! (
                "Alignment of field: " , stringify ! ( mp4AudioSpecificConfig
                ) , "::" , stringify ! ( downSampledSBR ) ));
}
impl Clone for mp4AudioSpecificConfig {
    fn clone(&self) -> Self { *self }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct NeAACDecConfiguration {
    pub defObjectType: ::std::os::raw::c_uchar,
    pub defSampleRate: ::std::os::raw::c_ulong,
    pub outputFormat: ::std::os::raw::c_uchar,
    pub downMatrix: ::std::os::raw::c_uchar,
    pub useOldADTSFormat: ::std::os::raw::c_uchar,
    pub dontUpSampleImplicitSBR: ::std::os::raw::c_uchar,
}
#[test]
fn bindgen_test_layout_NeAACDecConfiguration() {
    assert_eq!(::std::mem::size_of::<NeAACDecConfiguration>() , 24usize ,
               concat ! ( "Size of: " , stringify ! ( NeAACDecConfiguration )
               ));
    assert_eq! (::std::mem::align_of::<NeAACDecConfiguration>() , 8usize ,
                concat ! (
                "Alignment of " , stringify ! ( NeAACDecConfiguration ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecConfiguration ) ) . defObjectType
                as * const _ as usize } , 0usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecConfiguration )
                , "::" , stringify ! ( defObjectType ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecConfiguration ) ) . defSampleRate
                as * const _ as usize } , 8usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecConfiguration )
                , "::" , stringify ! ( defSampleRate ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecConfiguration ) ) . outputFormat
                as * const _ as usize } , 16usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecConfiguration )
                , "::" , stringify ! ( outputFormat ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecConfiguration ) ) . downMatrix as
                * const _ as usize } , 17usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecConfiguration )
                , "::" , stringify ! ( downMatrix ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecConfiguration ) ) .
                useOldADTSFormat as * const _ as usize } , 18usize , concat !
                (
                "Alignment of field: " , stringify ! ( NeAACDecConfiguration )
                , "::" , stringify ! ( useOldADTSFormat ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecConfiguration ) ) .
                dontUpSampleImplicitSBR as * const _ as usize } , 19usize ,
                concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecConfiguration )
                , "::" , stringify ! ( dontUpSampleImplicitSBR ) ));
}
impl Clone for NeAACDecConfiguration {
    fn clone(&self) -> Self { *self }
}
pub type NeAACDecConfigurationPtr = *mut NeAACDecConfiguration;
#[repr(C)]
pub struct NeAACDecFrameInfo {
    pub bytesconsumed: ::std::os::raw::c_ulong,
    pub samples: ::std::os::raw::c_ulong,
    pub channels: ::std::os::raw::c_uchar,
    pub error: ::std::os::raw::c_uchar,
    pub samplerate: ::std::os::raw::c_ulong,
    pub sbr: ::std::os::raw::c_uchar,
    pub object_type: ::std::os::raw::c_uchar,
    pub header_type: ::std::os::raw::c_uchar,
    pub num_front_channels: ::std::os::raw::c_uchar,
    pub num_side_channels: ::std::os::raw::c_uchar,
    pub num_back_channels: ::std::os::raw::c_uchar,
    pub num_lfe_channels: ::std::os::raw::c_uchar,
    pub channel_position: [::std::os::raw::c_uchar; 64usize],
    pub ps: ::std::os::raw::c_uchar,
}
#[test]
fn bindgen_test_layout_NeAACDecFrameInfo() {
    assert_eq!(::std::mem::size_of::<NeAACDecFrameInfo>() , 104usize , concat
               ! ( "Size of: " , stringify ! ( NeAACDecFrameInfo ) ));
    assert_eq! (::std::mem::align_of::<NeAACDecFrameInfo>() , 8usize , concat
                ! ( "Alignment of " , stringify ! ( NeAACDecFrameInfo ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecFrameInfo ) ) . bytesconsumed as
                * const _ as usize } , 0usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecFrameInfo ) ,
                "::" , stringify ! ( bytesconsumed ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecFrameInfo ) ) . samples as *
                const _ as usize } , 8usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecFrameInfo ) ,
                "::" , stringify ! ( samples ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecFrameInfo ) ) . channels as *
                const _ as usize } , 16usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecFrameInfo ) ,
                "::" , stringify ! ( channels ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecFrameInfo ) ) . error as * const
                _ as usize } , 17usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecFrameInfo ) ,
                "::" , stringify ! ( error ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecFrameInfo ) ) . samplerate as *
                const _ as usize } , 24usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecFrameInfo ) ,
                "::" , stringify ! ( samplerate ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecFrameInfo ) ) . sbr as * const _
                as usize } , 32usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecFrameInfo ) ,
                "::" , stringify ! ( sbr ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecFrameInfo ) ) . object_type as *
                const _ as usize } , 33usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecFrameInfo ) ,
                "::" , stringify ! ( object_type ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecFrameInfo ) ) . header_type as *
                const _ as usize } , 34usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecFrameInfo ) ,
                "::" , stringify ! ( header_type ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecFrameInfo ) ) .
                num_front_channels as * const _ as usize } , 35usize , concat
                ! (
                "Alignment of field: " , stringify ! ( NeAACDecFrameInfo ) ,
                "::" , stringify ! ( num_front_channels ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecFrameInfo ) ) . num_side_channels
                as * const _ as usize } , 36usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecFrameInfo ) ,
                "::" , stringify ! ( num_side_channels ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecFrameInfo ) ) . num_back_channels
                as * const _ as usize } , 37usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecFrameInfo ) ,
                "::" , stringify ! ( num_back_channels ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecFrameInfo ) ) . num_lfe_channels
                as * const _ as usize } , 38usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecFrameInfo ) ,
                "::" , stringify ! ( num_lfe_channels ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecFrameInfo ) ) . channel_position
                as * const _ as usize } , 39usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecFrameInfo ) ,
                "::" , stringify ! ( channel_position ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const NeAACDecFrameInfo ) ) . ps as * const _
                as usize } , 103usize , concat ! (
                "Alignment of field: " , stringify ! ( NeAACDecFrameInfo ) ,
                "::" , stringify ! ( ps ) ));
}
#[link(name="faad")]
extern "C" {
    pub fn NeAACDecGetErrorMessage(errcode: ::std::os::raw::c_uchar)
     -> *mut ::std::os::raw::c_char;
}
#[link(name="faad")]
extern "C" {
    pub fn NeAACDecGetCapabilities() -> ::std::os::raw::c_ulong;
}
#[link(name="faad")]
extern "C" {
    pub fn NeAACDecOpen() -> NeAACDecHandle;
}
#[link(name="faad")]
extern "C" {
    pub fn NeAACDecGetCurrentConfiguration(hDecoder: NeAACDecHandle)
     -> NeAACDecConfigurationPtr;
}
#[link(name="faad")]
extern "C" {
    pub fn NeAACDecSetConfiguration(hDecoder: NeAACDecHandle,
                                    config: NeAACDecConfigurationPtr)
     -> ::std::os::raw::c_uchar;
}
#[link(name="faad")]
extern "C" {
    pub fn NeAACDecInit(hDecoder: NeAACDecHandle,
                        buffer: *mut ::std::os::raw::c_uchar,
                        buffer_size: ::std::os::raw::c_ulong,
                        samplerate: *mut ::std::os::raw::c_ulong,
                        channels: *mut ::std::os::raw::c_uchar)
     -> ::std::os::raw::c_long;
}
#[link(name="faad")]
extern "C" {
    pub fn NeAACDecInit2(hDecoder: NeAACDecHandle,
                         pBuffer: *mut ::std::os::raw::c_uchar,
                         SizeOfDecoderSpecificInfo: ::std::os::raw::c_ulong,
                         samplerate: *mut ::std::os::raw::c_ulong,
                         channels: *mut ::std::os::raw::c_uchar)
     -> ::std::os::raw::c_char;
}
#[link(name="faad")]
extern "C" {
    pub fn NeAACDecInitDRM(hDecoder: *mut NeAACDecHandle,
                           samplerate: ::std::os::raw::c_ulong,
                           channels: ::std::os::raw::c_uchar)
     -> ::std::os::raw::c_char;
}
#[link(name="faad")]
extern "C" {
    pub fn NeAACDecPostSeekReset(hDecoder: NeAACDecHandle,
                                 frame: ::std::os::raw::c_long);
}
#[link(name="faad")]
extern "C" {
    pub fn NeAACDecClose(hDecoder: NeAACDecHandle);
}
#[link(name="faad")]
extern "C" {
    pub fn NeAACDecDecode(hDecoder: NeAACDecHandle,
                          hInfo: *mut NeAACDecFrameInfo,
                          buffer: *mut ::std::os::raw::c_uchar,
                          buffer_size: ::std::os::raw::c_ulong)
     -> *mut ::std::os::raw::c_void;
}
#[link(name="faad")]
extern "C" {
    pub fn NeAACDecDecode2(hDecoder: NeAACDecHandle,
                           hInfo: *mut NeAACDecFrameInfo,
                           buffer: *mut ::std::os::raw::c_uchar,
                           buffer_size: ::std::os::raw::c_ulong,
                           sample_buffer: *mut *mut ::std::os::raw::c_void,
                           sample_buffer_size: ::std::os::raw::c_ulong)
     -> *mut ::std::os::raw::c_void;
}
#[link(name="faad")]
extern "C" {
    pub fn NeAACDecAudioSpecificConfig(pBuffer: *mut ::std::os::raw::c_uchar,
                                       buffer_size: ::std::os::raw::c_ulong,
                                       mp4ASC: *mut mp4AudioSpecificConfig)
     -> ::std::os::raw::c_char;
}