# UpdatePost

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **i32** | Reference ID | 
**created** | **String** | ISO format time representation | 
**header** | **String** | Leading header text, in bold color and monospace font | 
**title** | **String** | Title text in Futura font | 
**summary** | **String** | Tweet content in system Sans font | 
**mask** | [**models::Shape**](Shape.md) | Shape of the mask | 
**locale** | [**models::SupportedLocale**](SupportedLocale.md) | Target locale of the post | 
**trashed** | **bool** | Whether to not display this post | 
**cover** | Option<[**models::UpdatePostCover**](UpdatePostCover.md)> |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


