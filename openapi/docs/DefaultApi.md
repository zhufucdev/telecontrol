# \DefaultApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**gallery_id_delete**](DefaultApi.md#gallery_id_delete) | **DELETE** /gallery/{id} | Delete Gallery Item
[**gallery_id_get**](DefaultApi.md#gallery_id_get) | **GET** /gallery/{id} | Query Gallery Item
[**gallery_id_patch**](DefaultApi.md#gallery_id_patch) | **PATCH** /gallery/{id} | Patch Gallery Item
[**gallery_list_get**](DefaultApi.md#gallery_list_get) | **GET** /gallery/list | List Gallery
[**gallery_put**](DefaultApi.md#gallery_put) | **PUT** /gallery | Create Gallery Item
[**image_id_get**](DefaultApi.md#image_id_get) | **GET** /image/{id} | Query Image Metadata
[**image_list_get**](DefaultApi.md#image_list_get) | **GET** /image/list | List Images
[**image_post**](DefaultApi.md#image_post) | **POST** /image | Upload Image
[**image_put**](DefaultApi.md#image_put) | **PUT** /image | Assign Image to CDN Resource
[**update_id_delete**](DefaultApi.md#update_id_delete) | **DELETE** /update/{id} | Delete Update Post
[**update_id_get**](DefaultApi.md#update_id_get) | **GET** /update/{id} | Query Update Post
[**update_id_patch**](DefaultApi.md#update_id_patch) | **PATCH** /update/{id} | Patch Update Post
[**update_list_get**](DefaultApi.md#update_list_get) | **GET** /update/list | List Update Posts
[**update_put**](DefaultApi.md#update_put) | **PUT** /update | Create Update Post
[**update_template_get**](DefaultApi.md#update_template_get) | **GET** /update/template | Update Post Card Template



## gallery_id_delete

> serde_json::Value gallery_id_delete(id)
Delete Gallery Item



### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i32** | Item identifier | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[PostAuthKey](../README.md#PostAuthKey)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## gallery_id_get

> serde_json::Value gallery_id_get(id)
Query Gallery Item



### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i32** | Item identifier | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[PostAuthKey](../README.md#PostAuthKey)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## gallery_id_patch

> models::GalleryItem gallery_id_patch(id, gallery_id_patch_request)
Patch Gallery Item



### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i32** | Item identifier | [required] |
**gallery_id_patch_request** | [**GalleryIdPatchRequest**](GalleryIdPatchRequest.md) |  | [required] |

### Return type

[**models::GalleryItem**](GalleryItem.md)

### Authorization

[PostAuthKey](../README.md#PostAuthKey)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/octet-stream

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## gallery_list_get

> Vec<models::GalleryItem> gallery_list_get(locale, limit)
List Gallery



### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**locale** | Option<[**Vec<String>**](String.md)> | Accepted locales |  |
**limit** | Option<**i32**> | At most how many results to return |  |

### Return type

[**Vec<models::GalleryItem>**](GalleryItem.md)

### Authorization

[PostAuthKey](../README.md#PostAuthKey)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## gallery_put

> i32 gallery_put(gallery_put_request)
Create Gallery Item



### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**gallery_put_request** | [**GalleryPutRequest**](GalleryPutRequest.md) |  | [required] |

### Return type

**i32**

### Authorization

[PostAuthKey](../README.md#PostAuthKey)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/octet-stream

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## image_id_get

> models::Image image_id_get(id)
Query Image Metadata



### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i32** | Image identifier | [required] |

### Return type

[**models::Image**](Image.md)

### Authorization

[PostAuthKey](../README.md#PostAuthKey)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/octet-stream

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## image_list_get

> Vec<models::Image> image_list_get()
List Images



### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Image>**](Image.md)

### Authorization

[PostAuthKey](../README.md#PostAuthKey)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## image_post

> models::ImagePost201Response image_post(x_alt_text, x_file_name, content_length, body)
Upload Image

Upload image to the CDN via the server as proxy

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**x_alt_text** | **String** | Alernative text describing the image content | [required] |
**x_file_name** | **String** | Name of the original image file | [required] |
**content_length** | **i32** |  | [required] |
**body** | **std::path::PathBuf** |  | [required] |

### Return type

[**models::ImagePost201Response**](_image_post_201_response.md)

### Authorization

[PostAuthKey](../README.md#PostAuthKey)

### HTTP request headers

- **Content-Type**: application/octet-stream
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## image_put

> i32 image_put(image_put_request)
Assign Image to CDN Resource

Often used if the image was uploaded on client side

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**image_put_request** | [**ImagePutRequest**](ImagePutRequest.md) |  | [required] |

### Return type

**i32**

### Authorization

[PostAuthKey](../README.md#PostAuthKey)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_id_delete

> models::UpdatePost update_id_delete(id)
Delete Update Post

Note that this endpoint removes the post DIRECTLY without trashing it. Use the PATCH method in such scenario.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i32** |  | [required] |

### Return type

[**models::UpdatePost**](UpdatePost.md)

### Authorization

[PostAuthKey](../README.md#PostAuthKey)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_id_get

> models::UpdatePost update_id_get(id)
Query Update Post



### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i32** |  | [required] |

### Return type

[**models::UpdatePost**](UpdatePost.md)

### Authorization

[PostAuthKey](../README.md#PostAuthKey)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_id_patch

> models::UpdatePost update_id_patch(id, update_id_patch_request)
Patch Update Post



### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** |  | [required] |
**update_id_patch_request** | [**UpdateIdPatchRequest**](UpdateIdPatchRequest.md) |  | [required] |

### Return type

[**models::UpdatePost**](UpdatePost.md)

### Authorization

[PostAuthKey](../README.md#PostAuthKey)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/octet-stream

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_list_get

> Vec<models::UpdatePost> update_list_get(locale, limit)
List Update Posts

Get a list of availble updates (the top-most section in highlight page)

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**locale** | Option<[**Vec<String>**](String.md)> | Accepted locale names |  |
**limit** | Option<**i32**> | At most how many results to return |  |

### Return type

[**Vec<models::UpdatePost>**](UpdatePost.md)

### Authorization

[PostAuthKey](../README.md#PostAuthKey)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/octet-stream

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_put

> i32 update_put(update_put_request)
Create Update Post



### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**update_put_request** | [**UpdatePutRequest**](UpdatePutRequest.md) |  | [required] |

### Return type

**i32**

### Authorization

[PostAuthKey](../README.md#PostAuthKey)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_template_get

> String update_template_get()
Update Post Card Template

Variables are wrapped in ${...}.  - ${cover.src} - ${cover.alt} - ${header.leading} - ${header.tailing} - ${title} - ${summary}

### Parameters

This endpoint does not need any parameter.

### Return type

**String**

### Authorization

[PostAuthKey](../README.md#PostAuthKey)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/html

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

