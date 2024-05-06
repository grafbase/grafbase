// This is a generated file. It should not be edited manually.
//
// You can decide to commit this file or add it to your `.gitignore`.
//
// By convention, this module is imported as `@grafbase/generated`. To make this syntax possible,
// add a `paths` entry to your `tsconfig.json`.
//
//  "compilerOptions": {
//    "paths": {
//      "@grafbase/generated": ["./grafbase/generated"]
//    }
//  }

export type Schema = {
  'AnalyticsBillingResult': {
    __typename?: 'AnalyticsBillingResult';
    url: string;
    dimension_id: string;
  };
  'AnalyticsEventDataInput': {
    userAgent: string;
    ipAddress: string;
  };
  'AnalyticsImpressionDeviceType': | 'Desktop'| 'Tablet'| 'Phone';
  'AnalyticsImpressionInputParam': {
    key: string;
    value: string;
  };
  'AnalyticsMetaClickInput': {
    click_placement: string | null;
    device_height: number;
    device_width: number;
    filter_destination_id: string | null;
    filter_departure_port_id: string | null;
    filter_cruise_line_id: string | null;
    filter_ship_id: string | null;
    filter_port_id: string | null;
    filter_cruise_style_id: string | null;
    filter_cruise_length: string | null;
    meta_bonus_offer: number | null;
    meta_bonus_hover: number | null;
    result_position: string;
    section: string | null;
    sem_ad_headline_id: number | null;
    sem_entry_page: string | null;
    sem_entry_page_ref: string | null;
    source: number | null;
    template_id: number | null;
    template_name: string;
    template_params: string | null;
    total_vendors: number;
    variation: string | null;
    vendor_id: number;
    syndicate: string | null;
    vendor_position: number;
    widget_id: string;
    widget_name: string;
    sailing_id: number;
    provider_id: number | null;
    sponsored_listing: number | null;
    sponsored_listing_id: number | null;
    event_url: string;
    url: string;
    transaction_id: string;
    sponsored_featured_deal: number | null;
    sponsored_featured_deal_id: number | null;
    is_featured_listing: boolean | null;
    viewport: Schema['AnalyticsMetaDeviceType'] | null;
  };
  'AnalyticsMetaDeviceType': | 'SMALL_MOBILE'| 'MOBILE'| 'TABLET'| 'DESKTOP'| 'WIDESCREEN';
  'AnalyticsMutation': {
    __typename?: 'AnalyticsMutation';
    trackMetaClickEvent?: Schema['AnalyticsBillingResult'] | null;
    impression?: boolean;
  };
  'AnalyticsQuery': {
    __typename?: 'AnalyticsQuery';
    _service?: Schema['Analytics_Service'];
  };
  'AnalyticsTPixelInput': {
    tp_sid: string;
    tp_trfsrc: string;
    tp_ts: string;
    tp_uuid: string;
  };
  'Analytics_Service': {
    __typename?: 'Analytics_Service';
  /**
   * The sdl representing the federated service capabilities. Includes federation directives, removes federation types, and includes rest of full schema after schema directives have been applied
   */
    sdl: string | null;
  };
  /**
   * A date string, such as 2007-12-03, is compliant with the full-date format outlined in section 5.6 of the RFC 3339 profile of the ISO 8601 standard for the representation of dates and times using the Gregorian calendar.
   * 
   * This scalar is a description of the date, as used for birthdays for example. It cannot represent an instant on the timeline.
   */
  'Date': any;
  'DbAbTestVariations': {
    __typename?: 'DbAbTestVariations';
    id: number;
    name: string;
    min: number;
    max: number;
    status: number;
    ccVariationCpmValue: number;
  };
  'DbAbTests': {
    __typename?: 'DbAbTests';
    id: number;
    name: string;
    evar: number;
    status: number;
    urlSite: string | null;
    url_site: string | null;
    countryId: number | null;
    country_id: number | null;
    deviceAudience: string;
    device_audience: string;
    variationForUser?: Schema['DbAbTestVariations'];
    variations?: Array<Schema['DbAbTestVariations']>;
  };
  'DbAccountResponse': {
    __typename?: 'DbAccountResponse';
    success: boolean;
    errors?: Array<Schema['DbFieldError']> | null;
  };
  'DbAdRequestInput': {
    name: string;
    email: string;
    title: string | null;
    company: string | null;
    phone: string | null;
    address: string | null;
    city: string | null;
    state: string | null;
    postalCode: string | null;
    webAddress: string | null;
    areasOfInterest: string | null;
    budget: string | null;
    additionalInfo: string | null;
    recipients: Array<string>;
    recaptchaToken: string;
  };
  'DbAdRequestResponse': {
    __typename?: 'DbAdRequestResponse';
    message: string | null;
    success: boolean;
  };
  'DbAdminUsers': {
    __typename?: 'DbAdminUsers';
    id: number;
    fullName: string | null;
    headshotImageId: number | null;
    jobTitle: string | null;
    articles?: Array<Schema['DbArticles']>;
    snippets?: Array<Schema['DbShipSnippets']>;
  };
  'DbAdvertorialAdImage': {
    __typename?: 'DbAdvertorialAdImage';
    href: string | null;
    image: string | null;
  };
  'DbAdvertorialBanner': {
    __typename?: 'DbAdvertorialBanner';
    btnText: string | null;
    hasSubContent: string | null;
    subContent?: Schema['DbAdvertorialSections'] | null;
    href: string | null;
    image: string | null;
    title: string | null;
  };
  'DbAdvertorialBlock': {
    __typename?: 'DbAdvertorialBlock';
    copy: string | null;
    header: string | null;
    href: string | null;
    image?: Schema['DbAdvertorialImage'] | null;
    subHeader: string | null;
    isWeather: string | null;
    weatherKey: string | null;
  };
  'DbAdvertorialBlocks': {
    __typename?: 'DbAdvertorialBlocks';
    blockA?: Schema['DbAdvertorialBlock'] | null;
    blockB?: Schema['DbAdvertorialBlock'] | null;
    blockC?: Schema['DbAdvertorialBlock'] | null;
    blockD?: Schema['DbAdvertorialBlock'] | null;
    blockE?: Schema['DbAdvertorialBlock'] | null;
    title?: Schema['DbAdvertorialBlock'] | null;
  };
  'DbAdvertorialBorderBottom': {
    __typename?: 'DbAdvertorialBorderBottom';
    borderBottom: string | null;
  };
  'DbAdvertorialCallouts': {
    __typename?: 'DbAdvertorialCallouts';
    title: string | null;
    reviewLink: string | null;
    cruiseLink: string | null;
    tipLink: string | null;
    boardLink: string | null;
  };
  'DbAdvertorialContent': {
    __typename?: 'DbAdvertorialContent';
    title: string | null;
    text: string | null;
    description: string | null;
    href: string | null;
    image: string | null;
    type: string | null;
    showButton: boolean;
    buttonText: string | null;
    buttonLink: string | null;
    buttonColor: string | null;
    buttonBackgroundColor: string | null;
  };
  'DbAdvertorialCurrency': {
    __typename?: 'DbAdvertorialCurrency';
    name: string | null;
    symbol: string | null;
  };
  'DbAdvertorialDigioh': {
    __typename?: 'DbAdvertorialDigioh';
    surveyID: string | null;
    emailID: string | null;
  };
  'DbAdvertorialFacButton': {
    __typename?: 'DbAdvertorialFacButton';
    href: string | null;
    name: string | null;
    path: string | null;
    text: string | null;
  };
  'DbAdvertorialFooter': {
    __typename?: 'DbAdvertorialFooter';
    image: string | null;
    logo: string | null;
    logoHeight: string | null;
    mainDescription: string | null;
    subDescription: string | null;
    title: string | null;
  };
  'DbAdvertorialImage': {
    __typename?: 'DbAdvertorialImage';
    id: string | null;
    options?: Schema['DbAdvertorialOptions'] | null;
  };
  'DbAdvertorialImageGallery': {
    __typename?: 'DbAdvertorialImageGallery';
    header: string | null;
    images?: Array<Schema['DbAdvertorialContent']> | null;
  };
  'DbAdvertorialIntro': {
    __typename?: 'DbAdvertorialIntro';
    description: string | null;
    header: string | null;
    youtubeId: string | null;
  };
  'DbAdvertorialLogo': {
    __typename?: 'DbAdvertorialLogo';
    text: string | null;
    id: string | null;
  };
  'DbAdvertorialMedia': {
    __typename?: 'DbAdvertorialMedia';
    description: string | null;
    header: string | null;
    image: string | null;
    isSlideShow: string | null;
    linkText: string | null;
    href: string | null;
    logo?: Schema['DbAdvertorialLogo'] | null;
    mediaPosition: string | null;
    mediaPositon: string | null;
    review: string | null;
    slideShow?: Schema['DbAdvertorialSlideShow'] | null;
    title: string | null;
    titleBackground: boolean;
    titleBackgroundColor: string | null;
    titleBackgroundTextColor: string | null;
    positions: number | null;
  };
  'DbAdvertorialNavBtn': {
    __typename?: 'DbAdvertorialNavBtn';
    title: string | null;
    backgroundColor: string | null;
    linkTextColor: string | null;
    btns?: Array<Schema['DbAdvertorialContent']> | null;
  };
  'DbAdvertorialOptions': {
    __typename?: 'DbAdvertorialOptions';
    height: string | null;
    width: string | null;
  };
  'DbAdvertorialResponse': {
    __typename?: 'DbAdvertorialResponse';
    topBanner?: Schema['DbAdvertorialBorderBottom'] | null;
    blockHeader?: Schema['DbAdvertorialBorderBottom'] | null;
    primaryBgColor?: Schema['DbColor'] | null;
    primaryColor?: Schema['DbColor'] | null;
    secondaryColor?: Schema['DbColor'] | null;
    seeDetails?: Schema['DbColor'] | null;
    expandButton?: Schema['DbColor'] | null;
    ad?: Schema['DbAdvertorialAdImage'] | null;
    banners?: Array<Schema['DbAdvertorialBanner']> | null;
    blocks?: Schema['DbAdvertorialBlocks'] | null;
    blockAd?: Schema['DbAdvertorialBlock'] | null;
    blockF?: Schema['DbAdvertorialBlock'] | null;
    blockG?: Schema['DbAdvertorialBlock'] | null;
    blockSliver?: Schema['DbAdvertorialBlock'] | null;
    bottomShelf?: Schema['DbAdvertorialShelf'] | null;
    countdownEnd: string | null;
    currency?: Schema['DbAdvertorialCurrency'] | null;
    expandVideo: string | null;
    experienceRow?: Schema['DbExperienceRow'] | null;
    facButton?: Schema['DbAdvertorialFacButton'] | null;
    footer?: Schema['DbAdvertorialFooter'] | null;
    callout?: Schema['DbAdvertorialCallouts'] | null;
    digioh?: Schema['DbAdvertorialDigioh'] | null;
    hero?: Schema['DbAdvertorialMedia'] | null;
    horizontal?: Schema['DbHorizontalBlocks'] | null;
    imageGallery?: Schema['DbAdvertorialImageGallery'] | null;
    impressionPixel: string | null;
    intro?: Schema['DbAdvertorialIntro'] | null;
    main?: Schema['DbAdvertorialContent'] | null;
    middleHighlight?: Schema['DbAdvertorialMedia'] | null;
    nav?: Schema['DbAdvertorialNavBtn'] | null;
    rawCss: string | null;
    rawJs: string | null;
    reviews: Array<string> | null;
    reviewsHeader: string | null;
    secondaryShelf?: Schema['DbAdvertorialSecondaryShelf'] | null;
    socialLinks?: Array<Schema['DbAdvertorialSocialLink']> | null;
    sponsoredBy: string | null;
    sponsoredLogo: string | null;
    sponsoredLogoWidth: number | null;
    showSponsoredLogo: boolean;
    showReturnToSeasLogo: boolean;
    switches?: Schema['DbAdvertorialSwitches'] | null;
    taTitleImage: string | null;
    topShelf?: Schema['DbAdvertorialShelf'] | null;
    vertical?: Schema['DbAdvertorialVerticalBlocks'] | null;
    youtubeId: string | null;
    isImage: boolean;
    image: string | null;
    vendorsFlipclockCss: string | null;
    vendorsFlipclockMinJs: string | null;
  };
  'DbAdvertorialSecondaryShelf': {
    __typename?: 'DbAdvertorialSecondaryShelf';
    header: string | null;
    items?: Array<Schema['DbAdvertorialShelfRow']> | null;
  };
  'DbAdvertorialSection': {
    __typename?: 'DbAdvertorialSection';
    content?: Array<Schema['DbAdvertorialContent']> | null;
    type: string | null;
  };
  'DbAdvertorialSections': {
    __typename?: 'DbAdvertorialSections';
    sections?: Array<Schema['DbAdvertorialSection']> | null;
    title: string | null;
  };
  'DbAdvertorialShelf': {
    __typename?: 'DbAdvertorialShelf';
    mediaPosition: string | null;
    rows?: Array<Schema['DbAdvertorialShelfRow']> | null;
    sideByButtonBackgroundColor: string | null;
    sideByButtonTextColor: string | null;
    sideByTagColor: string | null;
    sideByHeaderColor: string | null;
  };
  'DbAdvertorialShelfMedia': {
    __typename?: 'DbAdvertorialShelfMedia';
    image: string | null;
    isImage: string | null;
    youtubeId: string | null;
  };
  'DbAdvertorialShelfRow': {
    __typename?: 'DbAdvertorialShelfRow';
    description: string | null;
    header: string | null;
    href: string | null;
    linkText: string | null;
    media?: Schema['DbAdvertorialShelfMedia'] | null;
    tag: string | null;
    image: string | null;
  };
  'DbAdvertorialSlideShow': {
    __typename?: 'DbAdvertorialSlideShow';
    autoPlay: string | null;
    slides: Array<string> | null;
    slideTransitionTime: string | null;
  };
  'DbAdvertorialSocialLink': {
    __typename?: 'DbAdvertorialSocialLink';
    href: string | null;
    network: string | null;
  };
  'DbAdvertorialSwitches': {
    __typename?: 'DbAdvertorialSwitches';
    comingSoon: string | null;
    countdownClock: string | null;
    experienceRow: string | null;
    footer: string | null;
  };
  'DbAdvertorialVerticalBlockItems': {
    __typename?: 'DbAdvertorialVerticalBlockItems';
    buttonText: string | null;
    descriptionPreview: string | null;
    href: string | null;
    image: string | null;
    price: string | null;
    title: string | null;
  };
  'DbAdvertorialVerticalBlocks': {
    __typename?: 'DbAdvertorialVerticalBlocks';
    blocks?: Array<Schema['DbAdvertorialVerticalBlockItems']> | null;
  };
  'DbAdvertorials': {
    __typename?: 'DbAdvertorials';
    id: number;
    title: string;
    status: boolean;
    cruiseLineId: number | null;
    countryId: number;
    syndicationId: number;
    slug: string;
    template: string;
    dateStart: string | null;
    dateEnd: string | null;
    guaranteedVisits: number;
    vendorName: string | null;
    updatedAt: Schema['DbDateTime'];
    advertorialAttributes?: Schema['DbAdvertorialResponse'] | null;
  };
  'DbAlsekCabinCategories': {
    __typename?: 'DbAlsekCabinCategories';
    id: number;
    alsekShipVersionId: number;
    categoryColor: string | null;
    categoryCode: string | null;
    categoryName: string | null;
    cabinTypeId: number | null;
    isMetaCategory: number | null;
    constituentCategoriesOnThisDeck: string | null;
    constituentCategoriesAnywhereOnShip: string | null;
    cabinClassCode: string | null;
    extendedCabinType: string | null;
    sortOrder: number | null;
    minimumOccupancy: number | null;
    maximumOccupancy: number | null;
    relatedCategories: string | null;
    minimumCabinAndBalconyArea: number | null;
    maximumBalconyArea: number | null;
    categoryIcon: string | null;
    shortDescription: string | null;
    fullDescription: string | null;
    smallPhoto: string | null;
    largePhoto: string | null;
    categoryFloorplan: string | null;
    virtualTourUrl: string | null;
    slug: string | null;
    averageMemberRating: number | null;
    totalMemberReviews: number | null;
  };
  'DbAnswer': {
    __typename?: 'DbAnswer';
    id: number;
    userId: number;
    user?: Schema['DbSsoUser'] | null;
  };
  'DbArticleBookingPhase': | 'pre'| 'post';
  'DbArticleImage': {
    __typename?: 'DbArticleImage';
    url: string | null;
    title: string | null;
  };
  'DbArticleInput': {
    status: boolean | null;
    adminNotes: string | null;
    primarySubjectId: number | null;
    primarySubjectReferenceId: number | null;
    slideshowId: number | null;
    noIndex: boolean | null;
    ignoreSlideShowTitle: boolean | null;
    isMigrated: boolean | null;
    bookingPhase: Schema['DbArticleBookingPhase'] | null;
    isNegative: boolean | null;
    adminUserId: number | null;
    sponsoredContentTarget: string | null;
    eventDate: string | null;
    expireDate: string | null;
    publishDate: string | null;
    startDate: string | null;
    type: Schema['DbArticleType'];
    shoreExcursionRelated: boolean | null;
    clientKey: string;
    syndicationId: number | null;
    toc: Schema['DbArticleToc'] | null;
    tocLayout: Schema['DbArticleTocLayout'] | null;
    lastUpdatedOn: string;
    articleVersionsId: number | null;
    snippets: Array<Schema['DbCreateArticleSnippetInput']>;
  };
  'DbArticleOrder': | 'recency'| 'popular';
  'DbArticleSnippetInput': {
    articleId: number;
    popularity: number;
  };
  'DbArticleSnippets': {
    __typename?: 'DbArticleSnippets';
    snippetHeader: string | null;
    snippet_header: string | null;
    snippet: string;
    sortOrder: number | null;
    sort_order: number | null;
    countryId: number | null;
    country_id: number | null;
    showRelatedContent: boolean;
    show_related_content: boolean;
  };
  'DbArticleToc': | 'off'| 'bullet'| 'number';
  'DbArticleTocLayout': | 'none'| 'wrap';
  'DbArticleType': | 'article'| 'slideshow'| 'news';
  'DbArticleVersions': {
    __typename?: 'DbArticleVersions';
    id: number;
    publishedArticleId: number | null;
    status: number | null;
    eventDate: string | null;
    publishDate: string | null;
    startDate: Schema['DbDateTime'] | null;
    lastUpdatedOn: string | null;
  };
  'DbArticles': {
    __typename?: 'DbArticles';
    id: number;
    status: number | null;
    updatedAt: Schema['DbDateTime'];
    updated_at: Schema['DbDateTime'];
    primarySubjectId: number | null;
    primary_subject_id: number | null;
    primarySubjectReferenceId: number | null;
    primary_subject_reference_id: number | null;
    slideshowId: number | null;
    slideshow_id: number | null;
    noIndex: number | null;
    no_index: number | null;
    createdAt: Schema['DbDateTime'];
    created_at: Schema['DbDateTime'];
    isMigrated: boolean | null;
    is_migrated: number | null;
    bookingPhase: string | null;
    booking_phase: string | null;
    isNegative: boolean | null;
    is_negative: number | null;
    adminUserId: number | null;
    admin_user_id: number | null;
    sponsoredContentTarget: string | null;
    sponsored_content_target: string | null;
    eventDate: string | null;
    event_date: string | null;
    lastUpdatedOn: string | null;
    publishDate: string | null;
    publish_date: string | null;
    startDate: Schema['DbDateTime'] | null;
    type: string;
    clientKey: string;
    client_key: string;
    syndicationId: number;
    syndication_id: number;
    toc: string;
    isNews: boolean;
    articleVersionsId: number;
    slug: string | null;
    articleDate: string | null;
    subjects?: Array<Schema['DbISubject']> | null;
    title?: string;
    description?: string;
    promo?: string;
    body?: Array<Schema['DbArticleSnippets']>;
    articleVersion?: Schema['DbArticleVersions'];
    adminUsers?: Array<Schema['DbAdminUsers']> | null;
    articleImage?: Schema['DbArticleImage'] | null;
    countries?: Array<Schema['DbCountries']>;
    popularity?: number | null;
    breadcrumbs?: Array<Schema['DbISubject']> | null;
    seo?: Array<Schema['DbSeo']> | null;
  };
  'DbAuthInput': {
    password: string;
    email: string;
  };
  'DbBillingCurrencies': {
    __typename?: 'DbBillingCurrencies';
    id: number;
    name: string;
    symbol: string;
  };
  'DbBucketedCountryMapping': {
    __typename?: 'DbBucketedCountryMapping';
    type: Schema['DbCountryMappingType'];
    bucketedCountryCode: Schema['DbCountryCode'] | null;
    bucketedCountry?: Schema['DbCountries'] | null;
  };
  'DbBulkSnippetUpdate': {
    snippets: Array<Schema['DbArticleSnippetInput']>;
  };
  'DbCabinCategories': {
    __typename?: 'DbCabinCategories';
    id: number;
    categoryName: string | null;
    categoryCode: string | null;
    imageUrl: string | null;
    categoryColor: string | null;
    description: string | null;
    slug: string | null;
    averageMemberRating: number | null;
    totalMemberReviews: number | null;
  };
  'DbCabinCategoriesUnion': | Schema['DbCabinCategories'] | Schema['DbAlsekCabinCategories'];
  'DbCabinCategory': {
    __typename?: 'DbCabinCategory';
    id: number;
    cabinTypeId: number | null;
    categoryName: string | null;
    categoryCode: string | null;
    imageUrl: string | null;
    categoryColor: string | null;
    description: string | null;
    shipId: number | null;
    slug: string | null;
    averageMemberRating: number | null;
    totalMemberReviews: number | null;
    providerId: number | null;
    shipVersion: number | null;
    decks?: Array<Schema['DbDeck']> | null;
  };
  'DbCabinDetail': {
    __typename?: 'DbCabinDetail';
    cabinTypeId: number | null;
    sizeMin: number | null;
    sizeMax: number | null;
    connected: number | null;
    accessible: number | null;
    passengers: number | null;
    total: number | null;
  };
  'DbCabinDetailResponse': {
    __typename?: 'DbCabinDetailResponse';
    shipId: number | null;
    balcony?: Schema['DbCabinDetail'] | null;
    inside?: Schema['DbCabinDetail'] | null;
    outside?: Schema['DbCabinDetail'] | null;
    suite?: Schema['DbCabinDetail'] | null;
  };
  'DbCabinTypes': {
    __typename?: 'DbCabinTypes';
    id: number;
    name: string | null;
    slug: string | null;
  };
  'DbCheckPriceVendors': {
    __typename?: 'DbCheckPriceVendors';
    id: number;
    billingCurrencyId: number;
    cpc?: string;
    billingCurrency: string;
  };
  'DbColor': {
    __typename?: 'DbColor';
    backgroundColor: string | null;
    color: string | null;
  };
  'DbContentBlockItems': {
    __typename?: 'DbContentBlockItems';
    id: number;
    contentBlockId: string;
    content_block_id: string;
    key: string;
    value: string;
    countryId: number;
    country_id: number;
  };
  'DbContentBlocks': {
    __typename?: 'DbContentBlocks';
    id: number;
    name: string;
    updatedAt: Schema['DbDateTime'];
    updated_at: Schema['DbDateTime'];
  };
  'DbContentHub': {
    __typename?: 'DbContentHub';
    id: number;
    urlSlug: string;
    route: string;
    title: string;
    keyTargeting: string | null;
    metaKeywords: string | null;
    metaDescription: string | null;
    countryId: number;
    primarySubjectId: number | null;
    primarySubjectReferenceId: number | null;
    headlineTitle: string;
    populationType: Schema['DbContentHubPopulationType'];
    status: number;
    syndicationId: number;
    topContentHero: string | null;
    topContentVideo: string | null;
    topContentBody: string | null;
    topContentHeadline: string | null;
    primaryHeadLine: string | null;
    primaryPhoto: string | null;
    primaryBody: string | null;
    primaryLink: string | null;
    listOrder: string | null;
    listType: Schema['DbListType'] | null;
    createdAt: Schema['DbDateTime'];
    contentHubBlocks?: Array<Schema['DbContentHubBlocks']> | null;
    contentHubArticles?: Array<Schema['DbContentHubArticles']> | null;
    subjects?: Array<Schema['DbISubject']> | null;
  };
  'DbContentHubArticles': {
    __typename?: 'DbContentHubArticles';
    id: number;
    contentHubId: number;
    articleId: number;
  };
  'DbContentHubBlockInput': {
    contentHubId: number;
    title: string;
    imageUrl: string;
    body: string;
    linkUrl: string | null;
    sortOrder: number;
  };
  'DbContentHubBlocks': {
    __typename?: 'DbContentHubBlocks';
    id: number;
    contentHubId: number;
    title: string;
    imageUrl: string;
    body: string;
    linkUrl: string | null;
    sortOrder: number;
  };
  'DbContentHubInput': {
    urlSlug: string;
    title: string;
    keyTargeting: string | null;
    metaKeywords: string | null;
    metaDescription: string | null;
    countryId: number | null;
    primarySubjectId: number;
    primarySubjectReferenceId: number;
    headlineTitle: string | null;
    populationType: Schema['DbContentHubPopulationType'] | null;
    status: number | null;
    syndicationId: number | null;
    topContentHero: string | null;
    topContentVideo: string | null;
    topContentBody: string | null;
    topContentHeadline: string | null;
    primaryHeadLine: string | null;
    primaryPhoto: string | null;
    primaryBody: string | null;
    primaryLink: string | null;
    listOrder: string | null;
    listType: Schema['DbListType'] | null;
  };
  'DbContentHubPopulationType': | 'manual'| 'tagging'| 'recency'| 'popular';
  'DbContentHubRelationshipSubjectInput': {
    subjectId: number;
    subjectReferenceId: number;
  };
  'DbCountries': {
    __typename?: 'DbCountries';
    id: number;
    name: string;
    shortName: string;
    short_name: string;
    domain: string | null;
    override: number | null;
    status: number | null;
    hrefLanguage: string | null;
    href_language: string | null;
    billingCurrency?: Schema['DbBillingCurrencies'] | null;
  };
  'DbCountryCode': | 'US'| 'GB'| 'AU';
  'DbCountryMappingType': | 'domain'| 'meta';
  'DbCpcInput': {
    date: string;
    cruiseLine: number;
    ship: number;
    destination: number;
    section: string;
    ip: string | null;
    productId: number;
    sponsoredListingId: number | null;
    viewport: Schema['DbMetaDeviceType'];
  };
  'DbCreateArticleSnippetInput': {
    articleSnippetTitleId: number;
    snippetHeader: string | null;
    snippet: string;
    sortOrder: number | null;
    countryId: number | null;
    updatedAt: string;
    showRelatedContent: boolean;
  };
  'DbCruiseLineDeparturePort': {
    __typename?: 'DbCruiseLineDeparturePort';
    id: number;
    name: string | null;
    seoName: string | null;
  };
  'DbCruiseLineDestination': {
    __typename?: 'DbCruiseLineDestination';
    id: number;
    name: string | null;
    seoName: string | null;
  };
  'DbCruiseLinePartnerMessages': {
    __typename?: 'DbCruiseLinePartnerMessages';
    title: string;
    message: string | null;
    link: string | null;
    videoLink: string | null;
    authorName: string;
    authorPosition: string;
    authorAvatarImsId: number | null;
  };
  'DbCruiseLineShip': {
    __typename?: 'DbCruiseLineShip';
    id: number;
    name: string | null;
    seoName: string | null;
  };
  'DbCruiseLines': {
    __typename?: 'DbCruiseLines';
    id: number;
    main_name: string;
    short_name: string;
    name: string;
    status: number;
    imageUrl: string;
    image_url: string;
    slug: string;
    salesName: string | null;
    sales_name: string | null;
    shortName: string | null;
    logoUrl: string | null;
    logo_url: string | null;
    isLuxury: boolean;
    is_luxury: boolean;
    isRiver: boolean | null;
    is_river: number | null;
    isOwnYourOwn: boolean;
    iconUrl: string | null;
    seoName: string;
    seo_name: string;
    tier: string | null;
    logo: string;
    reviewName?: string;
    ships?: Array<Schema['DbShips']> | null;
    mainUrl: string | null;
    memberReviewUrl: string | null;
    partnerMessage?: Schema['DbCruiseLinePartnerMessages'] | null;
    snippets?: Schema['DbSubjectContentSnippets'];
    cruisersChoiceAwards?: Array<Schema['DbCruisersChoiceCategories']>;
    cruisersChoiceDestinationAwards?: Array<Schema['DbCruisersChoiceCategories']>;
    editorsPicksAwards?: Array<Schema['DbEditorsPicksCategories']>;
    editorsPicksResults?: Array<Schema['DbEditorsPicksResults']>;
    image?: string | null;
    isPopular?: boolean;
    totalReviewCount: number | null;
  };
  'DbCruiseLinesInput': {
    slug: string | null;
    isActiveForRollcalls: boolean | null;
  };
  'DbCruiseStyles': {
    __typename?: 'DbCruiseStyles';
    id: number;
    main_name: string;
    short_name: string;
    name: string;
    slug: string;
    iconUrl: string | null;
    icon_url: string | null;
    salesName: string;
    sales_name: string;
    forumId: number | null;
    forum_id: number | null;
    url: string | null;
    h1: string | null;
    h2: string | null;
    status: number;
    findACruiseId: number | null;
    find_a_cruise_id: number | null;
    seoName: string | null;
    seo_name: string | null;
    mainName: string;
    shortName: string;
    reviewName?: string;
  };
  'DbCruiseStylesInput': {
    id: Array<number> | null;
    slug: Array<string> | null;
    status: Array<boolean> | null;
  };
  'DbCruiseTips': {
    __typename?: 'DbCruiseTips';
    whatToKnow?: Array<Schema['DbArticles']>;
    destinationGuides?: Array<Schema['DbArticles']>;
    shipAndCruiseLine?: Array<Schema['DbArticles']>;
  };
  'DbCruisersChoiceCategories': {
    __typename?: 'DbCruisersChoiceCategories';
    id: number;
    name: string;
    title: string;
    section: string;
    year: number;
    countryId: number;
    position: number;
    subjectId: number;
    subject_reference_id: number | null;
    imageUrl: string | null;
    main_name: string;
    short_name: string;
    results?: Array<Schema['DbCruisersChoiceResults']>;
    subCategories: Array<string>;
  };
  'DbCruisersChoiceCategoriesInput': {
    name: string | null;
    year: number | null;
    section: string | null;
    countryId: number | null;
    subjectId: number | null;
  };
  'DbCruisersChoiceDestinationAwards': {
    __typename?: 'DbCruisersChoiceDestinationAwards';
    imageUrl: string | null;
    caption: string | null;
    size: string | null;
    userName: string | null;
    extraData: string | null;
    shipName: string | null;
    cruiseLineName: string | null;
    portName: string | null;
    portId: number | null;
    portSlug: string | null;
    categoryName: string | null;
    title: string | null;
  };
  'DbCruisersChoiceResults': {
    __typename?: 'DbCruisersChoiceResults';
    id: number;
    cruisersChoiceCategoryId: number;
    size: string;
    rating: number;
    totalReviews: number;
    imageUrl: string;
    caption: string;
    createdAt: Schema['DbDateTime'];
    updatedAt: Schema['DbDateTime'];
    isWinner: boolean;
    userName: string;
    extraData: string | null;
    subjectId: number;
    subjectReferenceId: number;
    type: string;
    port?: Schema['DbPorts'] | null;
    ship?: Schema['DbShips'] | null;
    cruiseLine?: Schema['DbCruiseLines'] | null;
  };
  /**
   * A date string, such as 2007-12-03, compliant with the `full-date` format outlined in section 5.6 of the RFC 3339 profile of the ISO 8601 standard for representation of dates and times using the Gregorian calendar.
   */
  'DbDate': any;
  /**
   * The javascript `Date` as string. Type represents date and time as the ISO Date string.
   */
  'DbDateTime': any;
  'DbDealAdvertisers': {
    __typename?: 'DbDealAdvertisers';
    id: number;
    name: string;
    hasAccess: number;
    countryId: number;
    isLuxury: number;
  };
  'DbDealNewsletterLinkInput': {
    id: number | null;
    advertiserId: number;
    firstName: string;
    lastName: string;
    phone: string;
    email: string;
    title: string;
    url: string;
    promoImageUrl: string;
    sailDate: string;
    destinationId: number;
    isTransatlantic: boolean;
    isWorldwide: boolean;
    shipId: number;
    countryId: number;
    sendEmail: boolean | null;
  };
  'DbDealNewsletterLinks': {
    __typename?: 'DbDealNewsletterLinks';
    id: number;
    advertiserId: number | null;
    advertiser?: Schema['DbDealAdvertisers'] | null;
    title: string | null;
    url: string | null;
    promoImageUrl: string | null;
    destinationId: number | null;
    destination?: Schema['DbDestinations'] | null;
    shipId: number | null;
    ship?: Schema['DbShips'] | null;
    sailDate: Schema['DbDateTime'] | null;
    submissionDate: Schema['DbDateTime'] | null;
    firstName: string | null;
    lastName: string | null;
    phoneNumber: string | null;
    emailAddress: string | null;
    isVerified: number | null;
    isArchived: number | null;
    isLuxury: number | null;
    startDate: Schema['DbDateTime'] | null;
    endDate: Schema['DbDateTime'] | null;
    countryId: number | null;
    ipAddress: string | null;
    status: number | null;
    isTransatlantic: boolean;
    isWorldwide: boolean;
  };
  'DbDealPromoInput': {
    dealPromoTypeId: number;
    title: string;
    snippet: string;
    url: string;
    promoImageUrl: string;
    logoImage: string | null;
    countryId: number;
    firstName: string;
    lastName: string;
    phone: string;
    email: string;
    sendEmail: boolean | null;
  };
  'DbDealPromos': {
    __typename?: 'DbDealPromos';
    id: number;
    dealPromoTypeId: number;
    title: string | null;
    snippet: string | null;
    url: string | null;
  };
  'DbDeck': {
    __typename?: 'DbDeck';
    id: number;
    name: string | null;
    number: number;
    imageUrl: string | null;
  };
  'DbDeparturePorts': {
    __typename?: 'DbDeparturePorts';
    id: number;
    main_name: string;
    short_name: string;
    name: string;
    portId: number | null;
    port_id: number | null;
    salesName: string | null;
    sales_name: string | null;
    seoName: string | null;
    seo_name: string | null;
    status: number;
    taLocationId: number | null;
    ta_location_id: number | null;
    destinations?: Array<Schema['DbDestinations']> | null;
    cruiseLines?: Array<Schema['DbCruiseLines']> | null;
    port?: Schema['DbPorts'] | null;
    slug: string | null;
    isIndexable: boolean;
    itineraryCount?: number | null;
  };
  'DbDestinations': {
    __typename?: 'DbDestinations';
    id: number;
    main_name: string;
    short_name: string;
    name: string;
    salesName: string | null;
    customDestinations: Array<number>;
    sales_name: string | null;
    status: number;
    destinationAreaId: number | null;
    destination_area_id: number | null;
    slug: string | null;
    imageUrl: string | null;
    image_url: string | null;
    forumId: number | null;
    forum_id: number | null;
    articleId: number | null;
    article_id: number | null;
    isRiver: boolean | null;
    is_river: number | null;
    seoName: string | null;
    seo_name: string | null;
    taLocationId: number | null;
    ta_location_id: number | null;
    mainName: string;
    shortName: string;
    reviewName?: string;
    ports?: Array<Schema['DbPorts']> | null;
    image?: string | null;
    overrideName?: string;
    ships?: Array<Schema['DbShips']> | null;
    naturalSeoName: string | null;
    memberReviewUrl: string | null;
    mainUrl: string | null;
    subjectContentSnippets?: Schema['DbSubjectContentSnippets'] | null;
  };
  'DbDeviceType': | 'DESKTOP_TABLET'| 'MOBILE';
  'DbEditorsPicksCategories': {
    __typename?: 'DbEditorsPicksCategories';
    id: number;
    categoryType: string;
    name: string;
    seoName: string | null;
    seo_name: string | null;
    sortOrder: number;
    countryId: number;
    year: number;
    result?: Schema['DbEditorsPicksResults'];
  };
  'DbEditorsPicksResults': {
    __typename?: 'DbEditorsPicksResults';
    name: string;
    description: string;
    imageUrl: string;
    subjectId: number;
    subjectReferenceId: number;
    category?: Schema['DbEditorsPicksCategories'] | null;
    ship?: Schema['DbShips'] | null;
    cruiseLine?: Schema['DbCruiseLines'] | null;
    port?: Schema['DbPorts'] | null;
  };
  'DbExperienceRow': {
    __typename?: 'DbExperienceRow';
    btnText: string | null;
  };
  'DbFacHeroImage': {
    __typename?: 'DbFacHeroImage';
    id: number;
    advertiserName: string;
    imageId: number;
    contentPosition: string;
    reviewSnippet: string;
    memberName: string;
    rating: number;
    readMoreLabel: string | null;
    url: string | null;
    impressionPixel: string | null;
  };
  'DbFacHeroImageFilters': {
    destinationId: Array<number> | null;
    cruiseLineId: Array<number> | null;
    portId: Array<number> | null;
  };
  'DbFeatures': {
    __typename?: 'DbFeatures';
    id: number;
    main_name: string;
    short_name: string;
    title: string | null;
    imageUrl: string | null;
    image_url: string | null;
    promoTitle: string | null;
    promo_title: string | null;
    promo: string | null;
    h1: string | null;
    h2: string | null;
    isFirstTimeCruiser: string | null;
    is_first_time_cruiser: string | null;
    mainName: string;
    shortName: string;
  };
  'DbFieldError': {
    __typename?: 'DbFieldError';
    path: string;
    message: string;
  };
  'DbFirstTimeCruisers': {
    __typename?: 'DbFirstTimeCruisers';
    id: number;
    main_name: string;
    short_name: string;
    title: string;
    mainName: string;
    shortName: string;
  };
  'DbGenericResponse': {
    __typename?: 'DbGenericResponse';
    message: string;
    success: boolean;
  };
  'DbHeroImage': {
    __typename?: 'DbHeroImage';
    imageId: number;
    description: string | null;
    imageDescription: string | null;
    imageTitle: string;
    position: string;
    rating: number | null;
    review?: Schema['DbReviews'] | null;
    advertorial?: Schema['DbHeroImageAdvertorial'] | null;
  };
  'DbHeroImageAdvertorial': {
    __typename?: 'DbHeroImageAdvertorial';
    author: string | null;
    rating: number | null;
    adUrl: string | null;
    attribution: string | null;
    label: string | null;
    pixel: string | null;
  };
  'DbHorizontalBlocks': {
    __typename?: 'DbHorizontalBlocks';
    blocks?: Array<Schema['DbAdvertorialContent']> | null;
  };
  'DbISubject': {
    id: number;
    main_name: string;
    short_name: string;
  };
  'DbImageMappings': {
    __typename?: 'DbImageMappings';
    id: number;
    subjectId: number;
    subjectReferenceId: number;
    imageUrl: string | null;
    title: string | null;
    identifier: string | null;
    sortOrder: number | null;
    imageId: number | null;
    countryId: number | null;
    image?: Schema['DbImages'] | null;
  };
  'DbImages': {
    __typename?: 'DbImages';
    id: number;
    type: string;
    prefix: string;
    slug: string | null;
    title: string | null;
    description: string | null;
    created_at: Schema['DbDateTime'];
    options: string | null;
  };
  'DbItineraries': {
    __typename?: 'DbItineraries';
    id: number;
    main_name: string;
    short_name: string;
    title: string | null;
    length: number | null;
    pastSailings?: Array<Schema['DbStoredSailings']>;
    departurePort?: Schema['DbPorts'];
    destination?: Schema['DbDestinations'];
    ports?: Array<Schema['DbPorts']>;
    hasMap: boolean;
  };
  'DbItineraryPort': {
    __typename?: 'DbItineraryPort';
    id: number;
    mappedImages?: Array<Schema['DbImages'] | null>;
  };
  'DbItineraryShip': {
    __typename?: 'DbItineraryShip';
    id: number;
    cruisersChoiceCategories?: Array<Schema['DbCruisersChoiceCategories']>;
    mappedImages?: Array<Schema['DbImages'] | null>;
    cruiseStyleIds: Array<number> | null;
  };
  'DbListItems': {
    __typename?: 'DbListItems';
    id: number;
    listId: number | null;
    description: string | null;
    url: string | null;
    subjectId: number | null;
    subjectReferenceId: number | null;
    sortOrder: number;
    countryId: number;
    status: number;
    extraData: string | null;
    imageUrl: string | null;
    title: string | null;
    image: string | null;
    destination?: Schema['DbDestinations'] | null;
    departurePort?: Schema['DbDeparturePorts'] | null;
    port?: Schema['DbPorts'] | null;
    ship?: Schema['DbShips'] | null;
    cruiseLine?: Schema['DbCruiseLines'] | null;
  };
  'DbListType': | 'article'| 'news';
  'DbLocale': | 'en_US'| 'en_UK'| 'en_AU';
  'DbMemberPhotoInput': {
    fileName: string;
    description: string;
    tags: Array<Schema['DbUserImageTag']> | null;
    portId: number | null;
  };
  'DbMetaDeviceType': | 'SMALL_MOBILE'| 'MOBILE'| 'TABLET'| 'DESKTOP'| 'WIDESCREEN';
  'DbMutation': {
    __typename?: 'DbMutation';
    createAdRequest?: Schema['DbAdRequestResponse'];
    updatePopularity?: Schema['DbStatusMessageResponse'];
    createArticle?: Schema['DbArticles'];
    updateArticle?: Schema['DbArticles'];
    deleteArticle?: Schema['DbArticles'];
    publishArticle?: boolean;
    updateArticleVersions?: Schema['DbArticleVersions'];
    register?: Schema['DbAccountResponse'];
    verify?: boolean;
    login?: Schema['DbUserResponse'];
    logout?: Schema['DbAccountResponse'];
    resetPassword?: Schema['DbAccountResponse'];
    forgotPassword?: Schema['DbAccountResponse'];
    changePassword?: Schema['DbAccountResponse'];
    updateContentHubBlock?: Schema['DbStatusMessageResponse'];
    createContentHubBlock?: Schema['DbStatusMessageResponse'];
    updateContentHub?: Schema['DbStatusMessageResponse'];
    createContentHub?: Schema['DbStatusMessageResponse'];
    deleteContentHubArticles?: Schema['DbStatusMessageResponse'];
    createContentHubArticles?: Schema['DbStatusMessageResponse'];
    createContentHubRelationships?: Schema['DbStatusMessageResponse'];
    upsertDealNewsletterLink?: Schema['DbGenericResponse'];
    createDealPromo?: Schema['DbGenericResponse'];
    submitFeedback?: Schema['DbSubmitFeedbackResponse'];
    newsletterSubscribe?: boolean;
    updateNewsletterSubscription?: boolean;
    addPollResult?: boolean;
    unsubscribeFromPriceAlert?: Schema['DbPriceAlertSubscriptions'] | null;
    addPriceAlertSubscription?: Schema['DbPriceAlertSubscriptionResponse'];
    reportUserImage?: boolean;
    addMemberPhoto?: boolean;
  };
  'DbNewsPromos': {
    __typename?: 'DbNewsPromos';
    id: number;
    title: string | null;
    snippet: string | null;
    url: string | null;
    imageUrl: string | null;
    relatedLinks: string | null;
    countryId: number | null;
    status: number | null;
  };
  'DbOverrideOwners': | 'findACruiseCheckPrices'| 'memberReviews'| 'firstTimeCruiser'| 'deal'| 'portName'| 'thirdPartyTraqFeed';
  'DbPackageType': | 'cruiseOnly'| 'cruiseAndHotel'| 'cruiseAndFlight'| 'notApplicable';
  'DbPackageTypes': {
    __typename?: 'DbPackageTypes';
    id: number;
    type: Schema['DbPackageType'];
    name: string;
  };
  'DbPointOfSale': {
    __typename?: 'DbPointOfSale';
    countryId: number;
    country: string;
    currency: string;
    currencySymbol: string;
  };
  'DbPollOptions': {
    __typename?: 'DbPollOptions';
    id: number;
    pollId: number | null;
    title: string | null;
    sortOrder: number | null;
  /**
   * Percentage of the total votes in the poll that are for this option
   */
    totalVotes: number | null;
  };
  'DbPolls': {
    __typename?: 'DbPolls';
    id: number;
    title: string | null;
    createdAt: Schema['DbDateTime'] | null;
    options?: Array<Schema['DbPollOptions']>;
    totalVotes: number;
  };
  'DbPopularityStats': {
    __typename?: 'DbPopularityStats';
    min: number;
    max: number;
  };
  'DbPorts': {
    __typename?: 'DbPorts';
    id: number;
    main_name: string;
    short_name: string;
    name: string;
    status: number;
    imageUrl: string | null;
    image_url: string | null;
    forumId: number | null;
    forum_id: number | null;
    slug: string;
    salesName: string | null;
    sales_name: string | null;
    destinationId: number | null;
    destination_id: number | null;
    longitude: string | null;
    latitude: string | null;
    showShoreExcursions: boolean | null;
    show_shore_excursions: number | null;
    showRestaurants: boolean | null;
    show_restaurants: boolean | null;
    isRiver: number | null;
    is_river: number | null;
    isPrivate: boolean | null;
    is_private: number | null;
    averageMemberRating: number | null;
    average_member_rating: number | null;
    totalMemberReviews: number | null;
    total_member_reviews: number | null;
    seoName: string | null;
    seo_name: string | null;
    tripadvisorId: number | null;
    tripadvisor_id: number | null;
    stateCode: string | null;
    state_code: string | null;
    countryCode: string | null;
    country_code: string | null;
    createdAt: Schema['DbDateTime'];
    created_at: Schema['DbDateTime'];
    updatedAt: Schema['DbDateTime'];
    updated_at: Schema['DbDateTime'];
    publishedAt: Schema['DbDateTime'] | null;
    published_at: Schema['DbDateTime'] | null;
    adminNotes: string | null;
    admin_notes: string | null;
    professionalOverallRating: string | null;
    professional_overall_rating: string | null;
    adminUserId: number | null;
    admin_user_id: number | null;
    taLocationId: number | null;
    ta_location_id: number | null;
    isPrimarilyDeparturePort: boolean;
    is_primarily_departure_port: number;
    reviewName?: string;
    destination?: Schema['DbDestinations'] | null;
    departurePort?: Schema['DbDeparturePorts'] | null;
    image?: string | null;
    naturalSeoName: string | null;
    memberReviewUrl: string | null;
    mainUrl: string | null;
    portOverviewUrl: string | null;
    shoreExcursionUrl: string | null;
    subjectContentSnippets?: Schema['DbSubjectContentSnippets'] | null;
    shoreExcursionSubjectContentSnippets?: Schema['DbSubjectContentSnippets'] | null;
    cruisersChoiceAwards?: Array<Schema['DbCruisersChoiceCategories']>;
    cruisersChoiceDestinationAwards?: Array<Schema['DbCruisersChoiceCategories']>;
    editorsPicksAwards?: Array<Schema['DbEditorsPicksCategories']>;
    editorsPicksResults?: Array<Schema['DbEditorsPicksResults']>;
    popularShoreExcursions?: Array<Schema['DbShoreExcursions']>;
    shoreExcursions?: Array<Schema['DbShoreExcursions']>;
    shoreExcursionCount: number;
    isPopularDeparturePort?: boolean;
    itineraryCount?: number | null;
  };
  'DbPriceAlertPrices': {
    __typename?: 'DbPriceAlertPrices';
    price: string;
  };
  'DbPriceAlertSubscribers': {
    __typename?: 'DbPriceAlertSubscribers';
    id: number;
    countryId: number;
    email: string;
    createdAt: Schema['DbDateTime'];
    subscriptions?: Array<Schema['DbPriceAlertSubscriptions']>;
  };
  'DbPriceAlertSubscriptionFilters': {
    __typename?: 'DbPriceAlertSubscriptionFilters';
    subjectId: number | null;
    subjectReferenceId: string | null;
  };
  'DbPriceAlertSubscriptionInput': {
    email: string | null;
    userId: number | null;
    countryId: number | null;
    cruiseLineId: number | null;
    shipId: number | null;
    portId: number | null;
    departurePortId: number | null;
    destinationId: number | null;
    cruiseStyleId: number | null;
    itineraryId: number | null;
    sailingId: number | null;
    month: string | null;
    city: string | null;
    region: string | null;
    country: string | null;
  };
  'DbPriceAlertSubscriptionResponse': {
    __typename?: 'DbPriceAlertSubscriptionResponse';
    success: boolean | null;
    message: string | null;
    subscriptionId: number | null;
  };
  'DbPriceAlertSubscriptions': {
    __typename?: 'DbPriceAlertSubscriptions';
    id: number;
    priceAlertSubscriberId: number;
    status: number;
    createdAt: Schema['DbDateTime'];
    updatedAt: Schema['DbDateTime'];
    name: string;
  };
  'DbPriceAlerts': {
    __typename?: 'DbPriceAlerts';
    sailingId: number;
    cabinTypeId: number;
    filters?: Array<Schema['DbPriceAlertSubscriptionFilters']>;
    prices?: Schema['DbPriceAlertPrices'];
    sailing?: Schema['DbStoredSailings'];
  };
  'DbQuery': {
    __typename?: 'DbQuery';
  /**
   */
    _entities?: Array<Schema['Db_Entity'] | null>;
    _service?: Schema['Db_Service'];
    abtest?: Schema['DbAbTests'] | null;
    abTests?: Array<Schema['DbAbTests']>;
    activeAdvertorials?: Array<Schema['DbAdvertorials']>;
    advertorial?: Schema['DbAdvertorials'] | null;
    showArticleNewsUrls?: Array<Schema['DbSitemapUrl']>;
    article?: Schema['DbArticles'] | null;
    articlesByIds?: Array<Schema['DbArticles']> | null;
    articles?: Array<Schema['DbArticles']>;
    news?: Array<Schema['DbArticles']>;
    cruiseTips?: Schema['DbCruiseTips'];
    articleSSGIds?: Array<number>;
    articlePopularityStats?: Schema['DbPopularityStats'];
    getUserFromResetHash?: Schema['DbUserResponse'];
    me?: Schema['DbUser'] | null;
    author?: Schema['DbAdminUsers'];
    authors?: Array<Schema['DbAdminUsers']>;
    shipCabinCategories?: Array<Schema['DbCabinCategory']> | null;
    cabinDetail?: Schema['DbCabinDetailResponse'];
    cabinType?: Schema['DbCabinTypes'] | null;
    cabinTypeBySlug?: Schema['DbCabinTypes'] | null;
    cpcFromCode?: string | null;
    urlHash?: string;
    checkPriceVendor?: Schema['DbCheckPriceVendors'];
    contentBlockItem?: Schema['DbContentBlockItems'] | null;
    contentBlockItems?: Array<Schema['DbContentBlockItems']> | null;
    contentBlockItemsByContentBlockId?: Array<Schema['DbContentBlockItems']> | null;
    contentBlock?: Schema['DbContentBlocks'] | null;
    contentBlocks?: Array<Schema['DbContentBlocks']> | null;
    contentHub?: Array<Schema['DbContentHub']>;
    contentHubUrls?: Array<Schema['DbSitemapUrl']>;
    countries?: Array<Schema['DbCountries']>;
    country?: Schema['DbCountries'];
    bucketedCountries?: Array<Schema['DbBucketedCountryMapping']>;
    cruiseLineDeparturePortName?: string | null;
    cruiseLineDeparturePortSeoName?: string | null;
    cruiseLineDestinationName?: string | null;
    cruiseLineDestinationSeoName?: string | null;
    cruisersChoiceAwards?: Array<Schema['DbCruisersChoiceCategories']> | null;
    cruisersChoiceDestinationAwards?: Array<Schema['DbCruisersChoiceCategories']> | null;
    cruisersChoiceCategoriesAwards?: Array<Schema['DbCruisersChoiceCategories']> | null;
    cruisersChoiceCategories?: Array<Schema['DbCruisersChoiceCategories']>;
    editorsPicksAwards?: Array<Schema['DbEditorsPicksCategories']> | null;
    editorsPicksCategories?: Array<Schema['DbEditorsPicksCategories']>;
    cruiseline?: Schema['DbCruiseLines'] | null;
    cruiselineBySlug?: Schema['DbCruiseLines'] | null;
    cruiselinesWithRollcalls?: Array<Schema['DbCruiseLines']>;
    cruiselines?: Array<Schema['DbCruiseLines']> | null;
    cruiseLinesByIds?: Array<Schema['DbCruiseLines']> | null;
    cruiseLineShipName?: string | null;
    cruiseLineShipSeoName?: string | null;
    cruiseStyle?: Schema['DbCruiseStyles'] | null;
    cruiseStyleBySlug?: Schema['DbCruiseStyles'] | null;
    cruiseStyles?: Array<Schema['DbCruiseStyles']> | null;
    cruiseStylesByIds?: Array<Schema['DbCruiseStyles']> | null;
    cruisersChoiceDestinationAwardsByCategory?: Array<Schema['DbCruisersChoiceDestinationAwards']> | null;
    dealAdvertiser?: Schema['DbDealAdvertisers'] | null;
    dealCountByIsLuxury?: number;
    dealNewsletterLink?: Schema['DbDealNewsletterLinks'] | null;
    dealNewsletterLinkFormEnabled?: boolean;
    dealNewsletterLinkFormOpenTimes?: string;
    dealNewsletterLinks?: Array<Schema['DbDealNewsletterLinks']>;
    dealPromo?: Schema['DbDealPromos'] | null;
    dealPromos?: Array<Schema['DbDealPromos']> | null;
    dealPromoFormEnabled?: boolean;
    dealPromoFormOpenTimes?: string;
    departurePort?: Schema['DbDeparturePorts'] | null;
    departurePortsByIds?: Array<Schema['DbDeparturePorts']> | null;
    departurePortsBySalesName?: Array<Schema['DbDeparturePorts']> | null;
    departurePortsBySearchTerm?: Array<Schema['DbDeparturePorts']> | null;
    allDeparturePorts?: Array<Schema['DbDeparturePorts']>;
    departurePorts?: Array<Schema['DbDeparturePorts']> | null;
    nearestDeparturePorts?: Array<Schema['DbDeparturePorts']> | null;
    destination?: Schema['DbDestinations'] | null;
    destinationBySlug?: Schema['DbDestinations'] | null;
    destinations?: Array<Schema['DbDestinations']> | null;
    destinationsByIds?: Array<Schema['DbDestinations']> | null;
    destinationsBySearchTerm?: Array<Schema['DbDestinations']> | null;
    destinationsPorts?: Array<Schema['DbPorts']> | null;
    destinationsImage?: string | null;
    destinationsOverrideName?: string;
    destinationsShips?: Array<Schema['DbShips']> | null;
    destinationsNaturalSeoName?: string | null;
    destinationsMemberReviewUrl?: string | null;
    destinationsMainUrl?: string | null;
    destinationsSubjectContentSnippets?: Schema['DbSubjectContentSnippets'] | null;
    facHeroImage?: Schema['DbFacHeroImage'] | null;
    feature?: Schema['DbFeatures'] | null;
    Features?: Array<Schema['DbFeatures']> | null;
    firstTimeCruiser?: Schema['DbFirstTimeCruisers'] | null;
    FirstTimeCruisers?: Array<Schema['DbFirstTimeCruisers']> | null;
    homeHeroImage?: Schema['DbHeroImage'] | null;
    heroImage?: Schema['DbHeroImage'] | null;
    imageMappings?: Array<Schema['DbImageMappings']>;
    mappedImages?: Array<Schema['DbImages'] | null>;
    mappedHeroImage?: Schema['DbHeroImage'] | null;
    itineraryPortMappedImages?: Array<Schema['DbImages'] | null>;
    storedItinerary?: Schema['DbItineraries'];
    storedItineraries?: Array<Schema['DbItineraries']>;
    itineraryShipCruisersChoiceCategories?: Array<Schema['DbCruisersChoiceCategories']>;
    itineraryShipCruiseStyleIds?: Array<number> | null;
    listItem?: Schema['DbListItems'] | null;
    listItems?: Array<Schema['DbListItems']> | null;
    latestNewsPromos?: Array<Schema['DbNewsPromos']>;
    packageType?: Schema['DbPackageTypes'];
    pointsOfSale?: Array<Schema['DbPointOfSale']>;
    polls?: Array<Schema['DbPolls']>;
    activePoll?: Schema['DbPolls'] | null;
    port?: Schema['DbPorts'] | null;
    portBySlug?: Schema['DbPorts'] | null;
    portsByIds?: Array<Schema['DbPorts']> | null;
    ports?: Array<Schema['DbPorts']> | null;
    priceAlert?: Schema['DbPriceAlerts'] | null;
    subscriber?: Schema['DbPriceAlertSubscribers'] | null;
    quizCruiseTags?: Array<Schema['DbQuizCruiseTags']>;
    recommendedItineraries?: Schema['DbRecommendedItineraries'];
    quizRecommendations?: Schema['DbRecommendedItineraries'];
    recommendedItinerariesBatch?: Schema['DbRecommendedItinerariesBatchResponse'];
    getRedirect?: Schema['DbRedirects'] | null;
    reviewByTotalHelpfulVotes?: number;
    reviewByTotalReviews?: number;
    reviewEntriesPort?: Schema['DbPorts'] | null;
    reviewEntriesShoreExcursion?: Schema['DbShoreExcursions'] | null;
    reviewComments?: Array<Schema['DbReviewComments']>;
    reviewStats?: Schema['DbReviewStats'] | null;
    reviewsCabinCategory?: Schema['DbCabinCategoriesUnion'] | null;
    reviewsImages?: Array<Schema['DbUserImages']>;
    reviewsUser?: Schema['DbSsoUser'] | null;
    reviewsComments?: Array<Schema['DbReviewComments']>;
    reviewsDeparturePort?: Schema['DbDeparturePorts'] | null;
    reviewsItinerary?: Schema['DbItineraries'] | null;
    reviewsDestinations?: Array<Schema['DbDestinations']> | null;
    sailingsByShip?: Array<Schema['DbStoredSailings']> | null;
    sailingFee?: Schema['DbSailingFees'] | null;
    searchAutocomplete?: Schema['DbSearchAutocompleteOptions'];
    shipAmenity?: Schema['DbShipAmenityResponse'];
    shipClass?: Schema['DbShipClasses'];
    ship?: Schema['DbShips'] | null;
    shipBySlug?: Schema['DbShips'] | null;
    shipsByIds?: Array<Schema['DbShips']> | null;
    ships?: Array<Schema['DbShips']> | null;
    shipsWithRollcalls?: Array<Schema['DbShips']>;
    newShips?: Array<Schema['DbShips']>;
    shipsShipClass?: Schema['DbShipClasses'] | null;
    shipsCruiseLine?: Schema['DbCruiseLines'] | null;
    shipsSlideshow?: Schema['DbSlideshows'] | null;
    shipsCleanSeoName?: string | null;
    shipsSeo?: Schema['DbSeo'] | null;
    shipsMappedImage?: Schema['DbImages'] | null;
    shipsImage?: string | null;
    shipsMappedImages?: Array<Schema['DbImageMappings']>;
    shipsSnippets?: Array<Schema['DbShipSnippets']> | null;
    shipsHasUserPhotos?: boolean;
    shipsHasItineraries?: boolean;
    shipsSnippetsForTypes?: Array<Schema['DbShipSnippets']> | null;
    shipsDeckPlanSlug?: string | null;
    shipsHasDeckPlans?: boolean;
    shipsDecks?: Array<Schema['DbDeck']> | null;
    shipsAttributes?: Schema['DbShipAttributes'] | null;
    shipsRatio?: string | null;
    shipsSize?: string | null;
    shipsAmenitiesByType?: Schema['DbShipAmenityResponse'] | null;
    shipsDestinations?: Array<Schema['DbDestinations']>;
    shipsDeparturePorts?: Array<Schema['DbDeparturePorts']> | null;
    shipsPorts?: Array<Schema['DbPorts']> | null;
    shipsPastSailings?: Array<Schema['DbStoredSailings']>;
    shipsAuthor?: Schema['DbAdminUsers'] | null;
    shipsCruisersChoiceAwards?: Array<Schema['DbCruisersChoiceCategories']>;
    shipsCruisersChoiceDestinationAwards?: Array<Schema['DbCruisersChoiceCategories']>;
    shipsEditorsPicksAwards?: Array<Schema['DbEditorsPicksCategories']>;
    shipsEditorsPicksResults?: Array<Schema['DbEditorsPicksResults']>;
    shipsCruiseStyles?: Array<Schema['DbCruiseStyles']>;
    shipsTotalShoreExcursions?: number;
    viatorUrl?: Schema['DbViatorUrl'] | null;
    shoreExcursionUrls?: Array<Schema['DbSitemapUrl']>;
    sponsoredContent?: Schema['DbSponsoredContent'] | null;
    ssoUser?: Array<Schema['DbSsoUser']> | null;
    travelLeadersGroupMappings?: Array<number>;
    userImages?: Array<Schema['DbUserImages']>;
    usersByIds?: Array<Schema['DbUsers']> | null;
    widget?: Schema['DbWidgets'] | null;
  };
  'DbQuestion': {
    __typename?: 'DbQuestion';
    id: number;
    userId: number;
    user?: Schema['DbSsoUser'] | null;
  };
  'DbQuizCruiseTags': {
    __typename?: 'DbQuizCruiseTags';
    id: number;
    tag: string;
    cruiseLineId: number | null;
    destinationId: number | null;
    isRiver: number | null;
  };
  'DbRecommendationSegment': {
    __typename?: 'DbRecommendationSegment';
    id: string;
    title: string | null;
  };
  'DbRecommendedItineraries': {
    __typename?: 'DbRecommendedItineraries';
    recommId: string;
    recommendations?: Array<Schema['DbRecommendedItinerary']>;
  };
  'DbRecommendedItinerariesBatch': {
    __typename?: 'DbRecommendedItinerariesBatch';
    scenario: string;
    segment?: Schema['DbRecommendationSegment'] | null;
    recommId: string;
    recommendations?: Array<Schema['DbRecommendedItinerary']>;
  };
  'DbRecommendedItinerariesBatchResponse': {
    __typename?: 'DbRecommendedItinerariesBatchResponse';
    responses?: Array<Schema['DbRecommendedItinerariesBatch']>;
  };
  'DbRecommendedItinerary': {
    __typename?: 'DbRecommendedItinerary';
    id: number;
    title: string;
    available: boolean;
    avgPricePerNight: number | null;
    country: Array<Schema['DbCountryCode']>;
    cruiseLineId: number;
    cruiseLineName: string;
    cruiseType: string;
    departureLatitude: string;
    departureLongitude: string;
    departurePortId: number;
    departurePortName: string;
    destinationIds: Array<number>;
    destinationNames: Array<string>;
    length: number;
    portIds: Array<number>;
    portNames: Array<string>;
    shipAverageMemberRating: number;
    shipId: number;
    shipMaidenYear: number;
    shipName: string;
    shipProMemberRating: number;
    shipSizeCategory: string;
    shipStyles: Array<string>;
    shipTotalMemberReviews: number;
    tags: Array<string>;
    type: string;
  };
  'DbRecommendedSegment': {
    maxSegments: number | null;
    lookupScenario: string;
    resultScenario: string;
  };
  'DbRedirects': {
    __typename?: 'DbRedirects';
    id: number;
    toUrl: string;
    fromUrl: string;
    countryId: number;
  };
  'DbReview': {
    __typename?: 'DbReview';
    id: string | null;
    comments?: Array<Schema['DbReviewComments']>;
  };
  'DbReviewBy': {
    __typename?: 'DbReviewBy';
    id: string | null;
    username: string | null;
    totalHelpfulVotes: number;
    totalReviews: number;
  };
  'DbReviewComments': {
    __typename?: 'DbReviewComments';
    id: number;
    reviewId: number;
    comment: string;
    status: number;
    commentBy: number;
    user?: Schema['DbUsers'];
  };
  'DbReviewEntries': {
    __typename?: 'DbReviewEntries';
    id: number;
    subjectId: number | null;
    subjectReferenceId: number | null;
    port?: Schema['DbPorts'] | null;
    shoreExcursion?: Schema['DbShoreExcursions'] | null;
  };
  'DbReviewSnippet': {
    __typename?: 'DbReviewSnippet';
    id: number;
    reviewId: number;
    snippet: string;
    categories?: Array<Schema['DbReviewsCategory']> | null;
  };
  'DbReviewSnippetsByCategory': {
    __typename?: 'DbReviewSnippetsByCategory';
    enrichmentActivities?: Array<Schema['DbReviewSnippet']> | null;
    valueForMoney?: Array<Schema['DbReviewSnippet']> | null;
    embarkation?: Array<Schema['DbReviewSnippet']> | null;
    dining?: Array<Schema['DbReviewSnippet']> | null;
    publicRooms?: Array<Schema['DbReviewSnippet']> | null;
    entertainment?: Array<Schema['DbReviewSnippet']> | null;
    cabin?: Array<Schema['DbReviewSnippet']> | null;
    fitnessAndRecreation?: Array<Schema['DbReviewSnippet']> | null;
    shoreExcursions?: Array<Schema['DbReviewSnippet']> | null;
    rates?: Array<Schema['DbReviewSnippet']> | null;
    underThree?: Array<Schema['DbReviewSnippet']> | null;
    threeToSix?: Array<Schema['DbReviewSnippet']> | null;
    sevenToNine?: Array<Schema['DbReviewSnippet']> | null;
    tenToTwelve?: Array<Schema['DbReviewSnippet']> | null;
    thirteenToFifteen?: Array<Schema['DbReviewSnippet']> | null;
    sixteenPlus?: Array<Schema['DbReviewSnippet']> | null;
    service?: Array<Schema['DbReviewSnippet']> | null;
    onboardExperience?: Array<Schema['DbReviewSnippet']> | null;
    family?: Array<Schema['DbReviewSnippet']> | null;
  };
  'DbReviewStats': {
    __typename?: 'DbReviewStats';
    count: number;
  };
  'DbReviewSummaries': {
    __typename?: 'DbReviewSummaries';
    id: number;
    summary: string;
  };
  'DbReviews': {
    __typename?: 'DbReviews';
    id: number;
    cabinCategoryCode: string | null;
    shipId: number | null;
    userId: string | null;
    imsId: string | null;
    embarkationPortId: number | null;
    destinationId: number | null;
    cruisedOn: Schema['DbDate'] | null;
    cruiseLength: number | null;
    cabinCategory?: Schema['DbCabinCategoriesUnion'] | null;
    images?: Array<Schema['DbUserImages']>;
    user?: Schema['DbSsoUser'] | null;
    comments?: Array<Schema['DbReviewComments']>;
    departurePort?: Schema['DbDeparturePorts'] | null;
    itinerary?: Schema['DbItineraries'] | null;
    destinations?: Array<Schema['DbDestinations']> | null;
  };
  'DbReviewsCategory': {
    __typename?: 'DbReviewsCategory';
    id: number;
    name: string;
  };
  'DbSailingFees': {
    __typename?: 'DbSailingFees';
    sailingId: number;
    cabinTypeId: number;
    countryId: number;
    providerId: number;
    fees: number;
    taxes: number;
  };
  'DbSearchAutocompleteLink': {
    __typename?: 'DbSearchAutocompleteLink';
    text: string;
    href: string;
  };
  'DbSearchAutocompleteOption': {
    __typename?: 'DbSearchAutocompleteOption';
    id: string;
    type: string;
    text: string;
    highlight: string;
    href: string;
    links?: Array<Schema['DbSearchAutocompleteLink']>;
    score: number;
  };
  'DbSearchAutocompleteOptions': {
    __typename?: 'DbSearchAutocompleteOptions';
    destination?: Array<Schema['DbSearchAutocompleteOption']>;
    oysterImage?: Array<Schema['DbSearchAutocompleteOption']>;
    article?: Array<Schema['DbSearchAutocompleteOption']>;
    port?: Array<Schema['DbSearchAutocompleteOption']>;
    ship?: Array<Schema['DbSearchAutocompleteOption']>;
    cruiseLine?: Array<Schema['DbSearchAutocompleteOption']>;
    page?: Array<Schema['DbSearchAutocompleteOption']>;
  };
  'DbSeo': {
    __typename?: 'DbSeo';
    metaTitle: string | null;
    meta_title: string | null;
    metaKeywords: string | null;
    meta_keywords: string | null;
    metaDescription: string | null;
    meta_description: string | null;
    isNoindex: boolean | null;
    is_noindex: number | null;
  };
  'DbShipAmenity': {
    __typename?: 'DbShipAmenity';
    additionalFees: boolean | null;
    isDeckOnly: boolean | null;
    aggregateCount: number | null;
    title: string | null;
    subtitle: string | null;
  };
  'DbShipAmenityResponse': {
    __typename?: 'DbShipAmenityResponse';
    shipId: number;
    activity?: Array<Schema['DbShipAmenity']> | null;
    dining?: Array<Schema['DbShipAmenity']> | null;
    entertainment?: Array<Schema['DbShipAmenity']> | null;
    other?: Array<Schema['DbShipAmenity']> | null;
  };
  'DbShipAttributes': {
    __typename?: 'DbShipAttributes';
    id: number;
    shipId: number;
    totalCrew: string | null;
    totalDecks: string | null;
    maidenDate: string | null;
    tonnage: string | null;
    passengerCapacity: string | null;
    placeOfRegistry: string | null;
    cdcScore: string | null;
    viewerImageCount: number | null;
    launchDatetime: string | null;
    launchTimezone: string | null;
  };
  'DbShipClasses': {
    __typename?: 'DbShipClasses';
    id: number;
    name: string;
    cruiseLine?: Schema['DbCruiseLines'];
  };
  'DbShipSnippetTitle': | 'whyGo'| 'whatsNew'| 'overview'| 'author'| 'fellowPassengers'| 'dressCode'| 'gratuity'| 'cabins'| 'dining'| 'entertainment'| 'publicRooms'| 'fitnessRecreation'| 'family'| 'shoreExcursions'| 'enrichment'| 'service'| 'valueForMoney'| 'rates'| 'custom'| 'itineraries'| 'smallIntro'| 'inclusions'| 'exclusions'| 'highlights';
  'DbShipSnippets': {
    __typename?: 'DbShipSnippets';
    id: number;
    ship_id: number;
    ship_snippet_title_id: number;
    snippet: string;
    rating: number | null;
    updatedAt: Schema['DbDateTime'] | null;
  };
  'DbShips': {
    __typename?: 'DbShips';
    id: number;
    main_name: string;
    short_name: string;
    name: string;
    cruiseLineId: number;
    image_url: string;
    imageUrl: string;
    status: number;
    onSiteUntil: string | null;
    slug: string | null;
    sales_name: string | null;
    salesName: string | null;
    professional_overall_rating: string | null;
    professionalOverallRating: string | null;
    is_active_for_rollcalls: number;
    isActiveForRollcalls: boolean;
    average_member_rating: number | null;
    averageMemberRating: number | null;
    weighted_average_member_rating: number | null;
    weightedAverageMemberRating: number | null;
    total_member_reviews: number | null;
    totalMemberReviews: number | null;
    member_love_percentage: number | null;
    memberLovePercentage: number | null;
    review_status: number | null;
    reviewStatus: number | null;
    is_luxury: boolean;
    isLuxury: boolean;
    is_active_for_mobile: number | null;
    isActiveForMobile: boolean | null;
    is_river: number | null;
    isRiver: boolean | null;
    is_family: number;
    isFamily: boolean;
    popularity_score: number | null;
    popularityScore: number | null;
    seo_name: string | null;
    seoName: string | null;
    profile_layout: string;
    profileLayout: string;
    profile_layout_revert: string | null;
    profileLayoutRevert: string | null;
    is_disabled_for_meet_and_mingle: boolean;
    isDisabledForMeetAndMingle: boolean;
    alternate_member_love_percentage: number | null;
    alternateMemberLovePercentage: number | null;
    first_glimpse_publish_date: Schema['DbDateTime'] | null;
    firstGlimpsePublishDate: Schema['DbDateTime'] | null;
    full_review_publish_date: Schema['DbDateTime'] | null;
    fullReviewPublishDate: Schema['DbDateTime'] | null;
    is_disabled_for_review: number | null;
    isDisabledForReview: boolean | null;
    admin_notes: string | null;
    adminNotes: string | null;
    mainName: string;
    shortName: string;
    reviewName?: string;
    shipClass?: Schema['DbShipClasses'] | null;
    cruiseLine?: Schema['DbCruiseLines'] | null;
    slideshow?: Schema['DbSlideshows'] | null;
    cleanSeoName: string | null;
    seo?: Schema['DbSeo'] | null;
    mappedImage?: Schema['DbImages'] | null;
    image: string | null;
    mappedImages?: Array<Schema['DbImageMappings']>;
    snippets?: Array<Schema['DbShipSnippets']> | null;
    hasUserPhotos: boolean;
    hasItineraries: boolean;
    snippetsForTypes?: Array<Schema['DbShipSnippets']> | null;
    deckPlanSlug: string | null;
    hasDeckPlans: boolean;
    decks?: Array<Schema['DbDeck']> | null;
    attributes?: Schema['DbShipAttributes'] | null;
    ratio: string | null;
    size: string | null;
    amenitiesByType?: Schema['DbShipAmenityResponse'] | null;
    destinations?: Array<Schema['DbDestinations']>;
    departurePorts?: Array<Schema['DbDeparturePorts']> | null;
    ports?: Array<Schema['DbPorts']> | null;
    pastSailings?: Array<Schema['DbStoredSailings']>;
    author?: Schema['DbAdminUsers'] | null;
    cruisersChoiceAwards?: Array<Schema['DbCruisersChoiceCategories']>;
    cruisersChoiceDestinationAwards?: Array<Schema['DbCruisersChoiceCategories']>;
    editorsPicksAwards?: Array<Schema['DbEditorsPicksCategories']>;
    editorsPicksResults?: Array<Schema['DbEditorsPicksResults']>;
    cruiseStyles?: Array<Schema['DbCruiseStyles']>;
    totalShoreExcursions: number;
    reviewsSummary?: Schema['DbReviewSummaries'] | null;
    reviewSnippetsByCategory?: Schema['DbReviewSnippetsByCategory'];
    maidenDate: string | null;
    maidenYear: number | null;
    primaryImage?: Schema['MetaImage'] | null;
    categoryRating?: number | null;
    reviews?: Array<Schema['MetaSearchReviewsResults'] | null> | null;
  };
  'DbShipsOrder': | 'name'| 'publishDate';
  'DbShoreExcursions': {
    __typename?: 'DbShoreExcursions';
    id: number;
    name: string;
    overview: string | null;
    imageUrl: string | null;
    averageMemberRating: number | null;
    totalMemberReviews: number | null;
    imageId: number | null;
  };
  'DbShorexSortOrder': | 'name'| 'rating'| 'totalReviews';
  'DbSitemapUrl': {
    __typename?: 'DbSitemapUrl';
    url: string;
    lastModified: string;
  };
  'DbSlideshowSlides': {
    __typename?: 'DbSlideshowSlides';
    id: number;
    caption: string;
    imageUrl: string;
    image_url: string;
    sortOrder: number;
    sort_order: number;
    heading: string;
    subheading: string;
    isIntro: boolean;
    is_intro: number;
    country?: Schema['DbCountries'] | null;
    slideshow?: Schema['DbSlideshows'] | null;
  };
  'DbSlideshows': {
    __typename?: 'DbSlideshows';
    id: number;
    title: string;
    status: number;
    isGlobal: boolean;
    is_global: number;
    updatedAt: Schema['DbDateTime'];
    updated_at: Schema['DbDateTime'];
    slides?: Array<Schema['DbSlideshowSlides']>;
    subject?: Schema['DbSubjects'];
  };
  'DbSnippet': {
    __typename?: 'DbSnippet';
    snippetTitleId: number | null;
    heading: string | null;
    question: string | null;
    analyticsTarget: string | null;
    answerMarkdown: string | null;
    featured: boolean | null;
  };
  'DbSponsoredContent': {
    __typename?: 'DbSponsoredContent';
    id: number;
    urlSlug: string;
    title: string;
    sponsoredBy: string | null;
    sponsoredByUrl: string | null;
    keyTargeting: string | null;
    countryId: number;
    imageUrl: string;
    body: string | null;
    templateType: string;
    status: number;
    syndicationId: number;
    cruiseLineId: number | null;
    endDate: string | null;
    impressionPixel: string | null;
    sponsoredContentBlocks?: Array<Schema['DbSponsoredContentBlocks']> | null;
  };
  'DbSponsoredContentBlocks': {
    __typename?: 'DbSponsoredContentBlocks';
    id: number;
    sponsoredContentId: number;
    title: string;
    imageUrl: string;
    body: string;
    linkUrl: string | null;
    sortOrder: number;
  };
  'DbSsoUser': {
    __typename?: 'DbSsoUser';
    id: string;
    firstName: string | null;
    lastName: string | null;
    email: string;
    username: string;
    displayName: string;
  /**
   * User's age rounded down to the nearest decade (e.g. 54 -> 50)
   */
    age: number | null;
  };
  'DbStatusMessageResponse': {
    __typename?: 'DbStatusMessageResponse';
    message: string | null;
    success: boolean;
  };
  'DbStoredSailings': {
    __typename?: 'DbStoredSailings';
    id: number;
    main_name: string;
    short_name: string;
    status: number;
    itineraryId: number;
    departureDate: string;
    title: string | null;
    providerId: string;
    providerSailingId: number | null;
    minRevelexPrice: string | null;
    createdAt: string;
    updatedAt: string;
    mainName: string;
    shortName: string;
    storedItinerary?: Schema['DbItineraries'] | null;
  };
  'DbSubjectContentSnippets': {
    __typename?: 'DbSubjectContentSnippets';
    intro?: Array<Schema['DbSnippet']>;
    questionsAnswers?: Array<Schema['DbSnippet']>;
    prosAndCons?: Array<Schema['DbSnippet']> | null;
    amenities?: Array<Schema['DbSnippet']> | null;
  };
  'DbSubjectImageType': | 'PRIMARY'| 'PRIMARY_ACTIVITY'| 'PRIMARY_CABIN'| 'PRIMARY_COLLAGE'| 'PRIMARY_DINING'| 'PRIMARY_SPOTLIGHT'| 'PRIMARY_SQUARE_LOGO';
  'DbSubjectType': | 'destinations'| 'ports'| 'ships'| 'cruiseLines'| 'cruiseStyles'| 'departurePorts'| 'articles'| 'firstTimeCruisers'| 'features'| 'itineraries'| 'sailings'| 'contentHub'| 'contentHubId'| 'contentHubs'| 'healthAndSafety';
  'DbSubjects': {
    __typename?: 'DbSubjects';
    id: number;
    name: string;
    key: string;
  };
  'DbSubmitFeedbackInput': {
    siteSection: string;
    url: string;
    surveyResponse: string;
  };
  'DbSubmitFeedbackResponse': {
    __typename?: 'DbSubmitFeedbackResponse';
    success: boolean | null;
    errors: string | null;
  };
  'DbUpdateArticleInput': {
    status: boolean | null;
    adminNotes: string | null;
    primarySubjectId: number | null;
    primarySubjectReferenceId: number | null;
    slideshowId: number | null;
    noIndex: boolean | null;
    ignoreSlideShowTitle: boolean | null;
    isMigrated: boolean | null;
    bookingPhase: Schema['DbArticleBookingPhase'] | null;
    isNegative: boolean | null;
    adminUserId: number | null;
    sponsoredContentTarget: string | null;
    eventDate: string | null;
    expireDate: string | null;
    publishDate: string | null;
    startDate: string | null;
    type: Schema['DbArticleType'];
    shoreExcursionRelated: boolean | null;
    clientKey: string;
    syndicationId: number | null;
    toc: Schema['DbArticleToc'] | null;
    tocLayout: Schema['DbArticleTocLayout'] | null;
    lastUpdatedOn: string;
    snippets: Array<Schema['DbUpdateArticleSnippetInput']>;
  };
  'DbUpdateArticleSnippetInput': {
    articleSnippetTitleId: number;
    snippetHeader: string | null;
    snippet: string;
    sortOrder: number | null;
    countryId: number | null;
    updatedAt: string;
    showRelatedContent: boolean;
    id: number | null;
  };
  'DbUpdateArticleVersionsInput': {
    status: boolean | null;
    publishedArticleId: number | null;
    eventDate: string | null;
    expireDate: string | null;
    publishDate: string | null;
    startDate: string | null;
    lastUpdatedOn: string | null;
  };
  'DbUser': {
    __typename?: 'DbUser';
    id: string;
    username: string;
    email: string;
    dob: string;
    verified: boolean;
    requiresModeration: boolean;
    level: number;
  };
  'DbUserImageTag': | 'activity'| 'food'| 'cabin'| 'ship'| 'port'| 'shorex'| 'poolSpaFitness'| 'misc';
  'DbUserImageTags': {
    __typename?: 'DbUserImageTags';
    id: number;
    name: string;
    linkName: string | null;
  };
  'DbUserImages': {
    __typename?: 'DbUserImages';
    id: number;
    fileName: string;
    caption: string;
    allowOnAggregate: boolean;
    imageProxyUrl: string;
    sectionTag?: Schema['DbUserImageTags'] | null;
    imageUrl?: string | null;
    review?: Schema['DbReviews'] | null;
  };
  'DbUserResponse': {
    __typename?: 'DbUserResponse';
    user?: Schema['DbUser'] | null;
    errors?: Schema['DbFieldError'] | null;
    tokens?: Schema['DbUserTokens'] | null;
    debug: string | null;
  };
  'DbUserTokens': {
    __typename?: 'DbUserTokens';
    access: string | null;
    refresh: string | null;
  };
  'DbUsers': {
    __typename?: 'DbUsers';
    id: number;
    firstName: string | null;
    lastNameFirstLetter: string | null;
    title: string;
  };
  'DbViatorUrl': {
    __typename?: 'DbViatorUrl';
    url: string;
    checksum: string;
  };
  'DbWidgets': {
    __typename?: 'DbWidgets';
    id: number;
    type: string;
    data: string | null;
    subjectId: number | null;
    subjectReferenceId: number | null;
  };
  'Db_Any': any;
  'Db_Entity': | Schema['DbCountries'] | Schema['DbCabinTypes'] | Schema['DbPorts'] | Schema['DbDestinations'] | Schema['DbShips'] | Schema['DbCruiseLines'] | Schema['DbDeparturePorts'] | Schema['DbStoredSailings'] | Schema['DbItineraries'] | Schema['DbCruiseStyles'] | Schema['DbReviews'] | Schema['DbCabinCategories'] | Schema['DbReviewComments'] | Schema['DbUsers'] | Schema['DbPackageTypes'] | Schema['DbAnswer'] | Schema['DbCruiseLineDeparturePort'] | Schema['DbCruiseLineDestination'] | Schema['DbCruiseLineShip'] | Schema['DbItineraryPort'] | Schema['DbItineraryShip'] | Schema['DbQuestion'] | Schema['DbReview'] | Schema['DbReviewBy'] | Schema['DbReviewEntries'];
  'Db_Service': {
    __typename?: 'Db_Service';
  /**
   * The sdl representing the federated service capabilities. Includes federation directives, removes federation types, and includes rest of full schema after schema directives have been applied
   */
    sdl: string | null;
  };
  'MetaArticleType': | 'article'| 'slideshow'| 'news';
  'MetaBonusOfferCategorization': {
    __typename?: 'MetaBonusOfferCategorization';
    id: string | null;
    offerString: string | null;
    benefitType?: Schema['MetaProviderDealBenefitTypes'] | null;
  };
  'MetaBonusOfferItineraryFilter': {
    __typename?: 'MetaBonusOfferItineraryFilter';
    id: string;
    name: string;
    totalResults: number;
    lowestPrice: number | null;
  };
  'MetaBonusOffersItineraryFilterSet': {
    __typename?: 'MetaBonusOffersItineraryFilterSet';
    totalResults: number;
    results?: Array<Schema['MetaBonusOfferItineraryFilter']>;
  };
  'MetaBrandBox': {
    __typename?: 'MetaBrandBox';
    id: number | null;
    mainImageId: number | null;
    header: string | null;
    subHeader: string | null;
    bullets: Array<string | null> | null;
    vendor?: Schema['MetaBrandBoxVendor'];
    callToAction: string | null;
    url: string | null;
    impressionTracker: string | null;
  };
  'MetaBrandBoxVendor': {
    __typename?: 'MetaBrandBoxVendor';
    id: number | null;
    name: string | null;
    salesforceName: string | null;
    logoUrl: string | null;
    imageId: number | null;
  };
  'MetaCabinType': | 'inside'| 'outside'| 'balcony'| 'suite';
  'MetaCabinTypeAggsResult': {
    __typename?: 'MetaCabinTypeAggsResult';
    cabinTypeId: number;
    totalReviews: number;
    averageMemberRating: number;
  };
  'MetaCabinTypeItineraryFilter': {
    __typename?: 'MetaCabinTypeItineraryFilter';
    id: string;
    name: string;
    totalResults: number;
    lowestPrice: number | null;
  };
  'MetaCabinTypeItineraryFilterSet': {
    __typename?: 'MetaCabinTypeItineraryFilterSet';
    totalResults: number;
    results?: Array<Schema['MetaCabinTypeItineraryFilter']>;
  };
  'MetaCabinTypes': {
    __typename?: 'MetaCabinTypes';
    id: number;
  };
  /**
   * How often a user has been on a cruise
   */
  'MetaCruiseExperienceLevel': | 'one'| 'couple'| 'few'| 'many';
  'MetaCruiseLineDeparturePort': {
    __typename?: 'MetaCruiseLineDeparturePort';
    id: number;
    name: string | null;
    seoName: string | null;
  };
  'MetaCruiseLineDestination': {
    __typename?: 'MetaCruiseLineDestination';
    id: number;
    name: string | null;
    seoName: string | null;
  };
  'MetaCruiseLineItineraryFilter': {
    __typename?: 'MetaCruiseLineItineraryFilter';
    id: string;
    name: string;
    totalResults: number;
    lowestPrice: number | null;
  };
  'MetaCruiseLineItineraryFilterSet': {
    __typename?: 'MetaCruiseLineItineraryFilterSet';
    totalResults: number;
    results?: Array<Schema['MetaCruiseLineItineraryFilter']>;
  };
  'MetaCruiseLineRelatedLinks': {
    __typename?: 'MetaCruiseLineRelatedLinks';
    ship?: Schema['MetaCruiseLineShip'];
    departurePorts?: Array<Schema['MetaCruiseLineDeparturePort']>;
    destinations?: Array<Schema['MetaCruiseLineDestination']>;
  };
  'MetaCruiseLineShip': {
    __typename?: 'MetaCruiseLineShip';
    id: number;
    name: string | null;
    seoName: string | null;
  };
  'MetaCruiseLineTier': | 'mainstream'| 'premium'| 'luxuryLite'| 'luxury';
  'MetaCruiseLines': {
    __typename?: 'MetaCruiseLines';
    id: number;
    totalReviewCount: number;
  };
  'MetaCruiseStyles': {
    __typename?: 'MetaCruiseStyles';
    id: number;
  };
  'MetaCurrency': | 'USD'| 'GBP'| 'AUD';
  /**
   * The javascript `Date` as string. Type represents date and time as the ISO Date string.
   */
  'MetaDateTime': any;
  'MetaDay': {
    __typename?: 'MetaDay';
    day: number;
    port?: Schema['MetaItineraryPort'];
  };
  'MetaDeal': {
    __typename?: 'MetaDeal';
    vendor?: Schema['MetaMetaVendor'];
    dropPercentage: number;
    pricePerNight: number;
    type: string;
    pricing?: Schema['MetaDealPricing'];
    cabinTypeId: number;
    dealType: string;
    pricePerNightFormatted: string;
  };
  'MetaDealItineraryFilter': {
    __typename?: 'MetaDealItineraryFilter';
    id: string;
    name: string;
    totalResults: number;
    lowestPrice: number | null;
  };
  'MetaDealItineraryFilterSet': {
    __typename?: 'MetaDealItineraryFilterSet';
    totalResults: number;
    results?: Array<Schema['MetaDealItineraryFilter']>;
  };
  'MetaDealPricing': {
    __typename?: 'MetaDealPricing';
    highestPrice14: number;
    sponsoredListingUrls: Array<string>;
    boostFactorCpc: number | null;
    price: number;
    sponsoredFeaturedDealUrls: Array<string>;
    cpc: string;
    pricePerNight: number;
    highestPrice: number;
    bonusOffers: Array<string>;
    cabinType?: Schema['MetaMetaCabinType'];
    packageType: Schema['MetaPackageType'];
    url: string;
  };
  'MetaDealStats': {
    __typename?: 'MetaDealStats';
    maxDropPercentage: number;
  };
  'MetaDealType': | 'all'| 'lastMinute';
  'MetaDeals': {
    __typename?: 'MetaDeals';
    totalResults: number;
    results?: Array<Schema['MetaDeal']>;
  };
  'MetaDepartureMonthItineraryFilter': {
    __typename?: 'MetaDepartureMonthItineraryFilter';
    id: string;
    name: string;
    totalResults: number;
    lowestPrice: number | null;
  };
  'MetaDepartureMonthItineraryFilterSet': {
    __typename?: 'MetaDepartureMonthItineraryFilterSet';
    totalResults: number;
    results?: Array<Schema['MetaDepartureMonthItineraryFilter']>;
  };
  'MetaDeparturePortItineraryFilter': {
    __typename?: 'MetaDeparturePortItineraryFilter';
    id: string;
    name: string;
    totalResults: number;
    lowestPrice: number | null;
  };
  'MetaDeparturePortItineraryFilterSet': {
    __typename?: 'MetaDeparturePortItineraryFilterSet';
    totalResults: number;
    results?: Array<Schema['MetaDeparturePortItineraryFilter']>;
  };
  'MetaDeparturePorts': {
    __typename?: 'MetaDeparturePorts';
    id: number;
    itineraryCount?: number;
  };
  'MetaDestinationItineraryFilter': {
    __typename?: 'MetaDestinationItineraryFilter';
    id: string;
    name: string;
    totalResults: number;
    lowestPrice: number | null;
  };
  'MetaDestinationItineraryFilterSet': {
    __typename?: 'MetaDestinationItineraryFilterSet';
    totalResults: number;
    results?: Array<Schema['MetaDestinationItineraryFilter']>;
  };
  'MetaDestinations': {
    __typename?: 'MetaDestinations';
    id: number;
    ports?: Array<Schema['DbPorts'] | null> | null;
    image?: string | null;
    ships?: Array<Schema['DbShips'] | null> | null;
  };
  'MetaFacHero': {
    __typename?: 'MetaFacHero';
    id: number;
    advertiserName: string | null;
    imageId: number | null;
    contentPosition: string;
    reviewSnippet: string;
    memberName: string;
    rating: number;
    readMoreLabel: string | null;
    url: string | null;
    impressionPixel: string | null;
  };
  'MetaFields': | 'default'| 'sailings'| 'allSailings'| 'pricing'| 'lowestPricedSailing'| 'dealSailing'| 'cruisersChoice'| 'filters'| 'filtersOnly'| 'shipAttributes'| 'deals'| 'shipInclusions'| 'stats'| 'brandBox'| 'facHero'| 'lowestPriceFilters';
  'MetaFilterOrder': | 'totalResults'| 'alphabetical';
  'MetaFilterStats': {
    __typename?: 'MetaFilterStats';
    topLengthName: string;
    topDealIds: Array<string>;
    topLengthIds: Array<string>;
    departureMonths?: Array<Schema['MetaFilterStatsDepartureMonth']>;
    topDestinationIds: Array<string>;
    topCruiseStyleIds: Array<string>;
    topCruiseLinesIds: Array<string>;
    topShipIds: Array<string>;
    topDeparturePortsIds: Array<string>;
    topPortsIds: Array<string>;
  };
  'MetaFilterStatsDepartureMonth': {
    __typename?: 'MetaFilterStatsDepartureMonth';
    id: string;
    name: string;
  };
  'MetaImage': {
    __typename?: 'MetaImage';
    id: number;
    format: string;
  };
  'MetaItinerariesResult': {
    __typename?: 'MetaItinerariesResult';
    totalResults: number;
    results?: Array<Schema['MetaItinerary']> | null;
    filters?: Schema['MetaItineraryFilters'] | null;
    stats?: Schema['MetaItineraryStats'] | null;
    brandBox?: Schema['MetaBrandBox'] | null;
    facHero?: Schema['MetaFacHero'] | null;
    currency: Schema['MetaCurrency'];
  };
  'MetaItinerariesStats': {
    __typename?: 'MetaItinerariesStats';
    averageLength: number | null;
  };
  'MetaItinerary': {
    __typename?: 'MetaItinerary';
    id: number;
    title: string;
    length: number;
    isSponsored: boolean;
    sponsoredListingId: number | null;
    sponsoredVendorId: number | null;
    impressionPixel: string | null;
    score: number;
    sponsoredFeaturedDealId: number | null;
    ship?: Schema['MetaItineraryShip'];
    cruiseLine?: Schema['MetaItineraryCruiseLine'];
    destination?: Schema['MetaItineraryDestination'];
    arrivalPort?: Schema['MetaItineraryPort'];
    departurePort?: Schema['MetaItineraryPort'];
    itinerary?: Schema['MetaSchedule'];
    departurePorts?: Schema['MetaItineraryDeparturePorts'];
    sailings?: Schema['MetaSailings'] | null;
    allSailings?: Schema['MetaSailings'] | null;
    lowestPricedSailing?: Schema['MetaSailing'] | null;
    dealSailing?: Schema['MetaSailing'] | null;
    injectName: Schema['MetaItineraryInjectName'] | null;
  };
  'MetaItineraryAndSailings': {
    __typename?: 'MetaItineraryAndSailings';
    itinerary?: Schema['MetaItinerary'] | null;
    sailings?: Schema['MetaSailings'] | null;
  };
  'MetaItineraryCruiseLine': {
    __typename?: 'MetaItineraryCruiseLine';
    id: number;
    name: string | null;
    shortName: string | null;
    slug: string | null;
    iconUrl: string | null;
    logoUrl: string | null;
    tier: Schema['MetaCruiseLineTier'] | null;
    snippets?: Schema['MetaItineraryFieldSnippets'];
    image?: Schema['MetaItineraryFieldImage'] | null;
  };
  'MetaItineraryDeparturePort': {
    __typename?: 'MetaItineraryDeparturePort';
    id: number;
    name: string;
    portId: number;
  };
  'MetaItineraryDeparturePorts': {
    __typename?: 'MetaItineraryDeparturePorts';
    totalResults: number;
    results?: Array<Schema['MetaItineraryDeparturePort']>;
  };
  'MetaItineraryDestination': {
    __typename?: 'MetaItineraryDestination';
    id: number;
    name: string;
    seoName: string;
    imageUrl: string;
    image?: Schema['MetaItineraryFieldImage'];
    taLocationId: number | null;
  };
  'MetaItineraryFieldImage': {
    __typename?: 'MetaItineraryFieldImage';
    id: number;
    format: string;
    ratios: Array<number>;
  };
  'MetaItineraryFieldSnippets': {
    __typename?: 'MetaItineraryFieldSnippets';
    totalResults: number;
    results: Array<string>;
  };
  'MetaItineraryFilterInput': {
    deals: Schema['MetaDealType'] | null;
    length: Array<string> | null;
    sailingId: Array<number> | null;
    packageType: Array<Schema['MetaPackageType']> | null;
    departureDate: string | null;
    hideSoldOut: boolean | null;
    vendorIds: Array<number> | null;
    destinationId: Array<number> | null;
    itineraryId: number | null;
    cruiseLineId: Array<number> | null;
    shipId: Array<number> | null;
    portId: Array<number> | null;
    departurePortId: Array<number> | null;
    cruiseStyleId: Array<number> | null;
    departureDateEnd: string | null;
    departureDateInterval: number | null;
    minPrice: number | null;
    maxPrice: number | null;
    cabinType: Schema['MetaCabinType'] | null;
    cabinTypeId: number | null;
    bonusOfferIds: Array<string> | null;
    hasPricingForViewport: boolean | null;
    cruiseLineTier: Schema['MetaCruiseLineTier'] | null;
    hasPricingAvailable: number | null;
    includeSponsoredItinerary: boolean | null;
    lowestPriceNumberOfSailings: number | null;
    fields: Array<Schema['MetaFields']> | null;
  };
  'MetaItineraryFilters': {
    __typename?: 'MetaItineraryFilters';
    destinations?: Schema['MetaDestinationItineraryFilterSet'];
    lifestyles?: Schema['MetaLifestyleItineraryFilterSet'];
    cruiseLines?: Schema['MetaCruiseLineItineraryFilterSet'];
    ships?: Schema['MetaShipItineraryFilterSet'];
    departurePorts?: Schema['MetaDeparturePortItineraryFilterSet'];
    ports?: Schema['MetaPortItineraryFilterSet'];
    lengths?: Schema['MetaLengthItineraryFilterSet'];
    departureMonths?: Schema['MetaDepartureMonthItineraryFilterSet'];
    deals?: Schema['MetaDealItineraryFilterSet'];
    packageTypes?: Schema['MetaPackageTypeItineraryFilterSet'];
    cabinTypes?: Schema['MetaCabinTypeItineraryFilterSet'];
    bonusOffers?: Schema['MetaBonusOffersItineraryFilterSet'];
    price?: Schema['MetaPriceFilter'];
  };
  'MetaItineraryInjectName': | 'cheapest'| 'priceDrop'| 'lastMinute';
  'MetaItineraryPort': {
    __typename?: 'MetaItineraryPort';
    id: number;
    name: string;
    imageUrl: string | null;
    averageMemberRating: number;
    latitude: number | null;
    longitude: number | null;
    destinationId: number | null;
    mappedImages?: Array<Schema['DbImages'] | null> | null;
    cruisersChoiceCategories?: Array<Schema['DbCruisersChoiceCategories'] | null> | null;
  };
  'MetaItineraryShip': {
    __typename?: 'MetaItineraryShip';
    id: number;
    name: string;
    imageUrl: string;
    averageMemberRating: number;
    memberLovePercentage: number | null;
    totalMemberReviews: number;
    snippets?: Schema['MetaItineraryFieldSnippets'];
    image?: Schema['MetaItineraryFieldImage'] | null;
    inclusions?: Schema['MetaShipInclusions'] | null;
    cruiseStyleIds: Array<number | null> | null;
  };
  'MetaItinerarySortOrder': | 'popularity'| 'popularitySem'| 'popularityTest'| 'popularitySemTest'| 'popularityQuery'| 'popularityQuerySem'| 'popularityQueryV3'| 'popularityQuerySemV3'| 'score'| 'departureDate'| 'cruiseLine'| 'ship'| 'length'| 'rating'| 'price'| 'priceDesc';
  'MetaItineraryStats': {
    __typename?: 'MetaItineraryStats';
    filters?: Schema['MetaFilterStats'] | null;
    deals?: Schema['MetaDealStats'] | null;
    pricing?: Schema['MetaPricingStats'];
    pricingPerMonth?: Array<Schema['MetaPricingPerMonth']>;
    itineraries?: Schema['MetaItinerariesStats'] | null;
    isCached: boolean | null;
    cachedAt: Schema['MetaDateTime'] | null;
  };
  'MetaLengthItineraryFilter': {
    __typename?: 'MetaLengthItineraryFilter';
    id: string;
    name: string;
    totalResults: number;
    lowestPrice: number | null;
  };
  'MetaLengthItineraryFilterSet': {
    __typename?: 'MetaLengthItineraryFilterSet';
    totalResults: number;
    results?: Array<Schema['MetaLengthItineraryFilter']>;
  };
  'MetaLifestyleItineraryFilter': {
    __typename?: 'MetaLifestyleItineraryFilter';
    id: string;
    name: string;
    totalResults: number;
    lowestPrice: number | null;
  };
  'MetaLifestyleItineraryFilterSet': {
    __typename?: 'MetaLifestyleItineraryFilterSet';
    totalResults: number;
    results?: Array<Schema['MetaLifestyleItineraryFilter']>;
  };
  'MetaLowestPrice': {
    __typename?: 'MetaLowestPrice';
    key: string;
    itineraryId: number | null;
    sailingId: number | null;
    departureDate: string | null;
    price: string | null;
    highestPrice: string | null;
    currency: string | null;
    vendor?: Schema['MetaMetaVendor'];
  };
  'MetaLowestPriceWithVendor': {
    __typename?: 'MetaLowestPriceWithVendor';
    price?: Schema['MetaMetaPrice'];
    vendor?: Schema['MetaMetaVendor'];
  };
  'MetaLowestPrices': {
    __typename?: 'MetaLowestPrices';
    results?: Array<Schema['MetaLowestPrice']>;
  };
  'MetaLowestPricesFields': | 'default'| 'vendor';
  'MetaLowestPricesInput': {
    subjectId: number | null;
    partitionBy: Schema['MetaPartitionBy'] | null;
    partitionValues: Array<string>;
    length: Array<string> | null;
    sailingId: Array<number> | null;
    departureDate: string | null;
    destinationId: Array<number> | null;
    itineraryId: number | null;
    cruiseLineId: Array<number> | null;
    shipId: Array<number> | null;
    portId: Array<number> | null;
    departurePortId: Array<number> | null;
    cruiseStyleId: Array<number> | null;
    departureDateEnd: string | null;
    departureDateInterval: number | null;
    cabinType: Schema['MetaCabinType'] | null;
    cabinTypeId: number | null;
    fields: Array<Schema['MetaLowestPricesFields']> | null;
  };
  'MetaMeta': {
    __typename?: 'MetaMeta';
    currency: string;
    totalResults: number;
    results?: Array<Schema['MetaMetaItem']>;
  };
  'MetaMetaCabinType': {
    __typename?: 'MetaMetaCabinType';
    id: number;
    name: Schema['MetaSailingCabinType'];
  };
  'MetaMetaDeviceType': | 'SMALL_MOBILE'| 'MOBILE'| 'TABLET'| 'DESKTOP'| 'WIDESCREEN';
  'MetaMetaItem': {
    __typename?: 'MetaMetaItem';
    vendor?: Schema['MetaMetaVendor'];
    prices?: Schema['MetaMetaPrices'];
  };
  'MetaMetaPrice': {
    __typename?: 'MetaMetaPrice';
    cabinType?: Schema['MetaMetaCabinType'];
    price: string;
    priceFormatted: string;
    packageType: Schema['MetaPackageType'] | null;
    packageTypeFormatted: string | null;
    url: string;
    sponsoredFeatureDealUrls: Array<string>;
    bonusOffers?: Schema['MetaItineraryFieldSnippets'];
    highestPrice: string;
    highestPriceFormatted: string;
    pricePerNight: string;
    pricePerNightFormatted: string;
    deals?: Schema['MetaDeals'] | null;
  };
  'MetaMetaPrices': {
    __typename?: 'MetaMetaPrices';
    totalResults: number;
    results?: Array<Schema['MetaMetaPrice']>;
  };
  'MetaMetaVendor': {
    __typename?: 'MetaMetaVendor';
    id: number;
    name: string;
    imageUrl: string;
    phoneNumber: string | null;
    c2cTracker: string | null;
  };
  'MetaPackageType': | 'cruiseOnly'| 'cruiseAndHotel'| 'cruiseAndFlight'| 'notApplicable';
  'MetaPackageTypeItineraryFilter': {
    __typename?: 'MetaPackageTypeItineraryFilter';
    id: string;
    name: string;
    totalResults: number;
    lowestPrice: number | null;
  };
  'MetaPackageTypeItineraryFilterSet': {
    __typename?: 'MetaPackageTypeItineraryFilterSet';
    totalResults: number;
    results?: Array<Schema['MetaPackageTypeItineraryFilter']>;
  };
  'MetaPartitionBy': | 'cruiseLineId'| 'shipId'| 'portId'| 'departurePortId'| 'destinationId'| 'cruiseStyleId'| 'departureDate'| 'length'| 'itineraryId'| 'cabinType'| 'sailingId';
  'MetaPortItineraryFilter': {
    __typename?: 'MetaPortItineraryFilter';
    id: string;
    name: string;
    totalResults: number;
    lowestPrice: number | null;
  };
  'MetaPortItineraryFilterSet': {
    __typename?: 'MetaPortItineraryFilterSet';
    totalResults: number;
    results?: Array<Schema['MetaPortItineraryFilter']>;
  };
  'MetaPorts': {
    __typename?: 'MetaPorts';
    id: number;
    itineraryCount?: number;
  };
  'MetaPosCountry': | 'AU'| 'GB'| 'US';
  'MetaPriceFilter': {
    __typename?: 'MetaPriceFilter';
    min: number;
    max: number;
  };
  'MetaPricingPerMonth': {
    __typename?: 'MetaPricingPerMonth';
    departureMonth: string;
    totalResults: number;
    minPrice: number;
    minPriceFormatted: string;
  };
  'MetaPricingStats': {
    __typename?: 'MetaPricingStats';
    minPrice: number;
    minPriceFormatted: string | null;
    maxPrice: number;
    maxPriceFormatted: string;
    minPricePerNight: number;
    minPricePerNightFormatted: string;
    maxPricePerNight: number;
  };
  'MetaProviderDealBenefitTypes': {
    __typename?: 'MetaProviderDealBenefitTypes';
    id: string;
  };
  'MetaQuery': {
    __typename?: 'MetaQuery';
    _entities?: Array<Schema['Meta_Entity'] | null>;
    _service?: Schema['Meta_Service'];
    bonusOfferCategorization?: Array<Schema['MetaBonusOfferCategorization']> | null;
    cabinTypeAggs?: Array<Schema['MetaCabinTypeAggsResult']> | null;
    cruiseLineRelatedLinks?: Array<Schema['MetaCruiseLineRelatedLinks']> | null;
    totalCruiselineReviewCount?: number;
    departurePortsItineraryCount?: number;
    findACruiseUrls?: Array<Schema['MetaSitemapUrl']>;
    findACruiseDealUrls?: Array<Schema['MetaSitemapUrl']>;
    itineraryCount?: number;
    itineraryCounts?: Array<number>;
    itinerary?: Schema['MetaItinerary'] | null;
    itineraries?: Schema['MetaItinerariesResult'];
    sponsoredItineraries?: Schema['MetaSponsoredItinerariesResult'];
    lowestPrices?: Schema['MetaLowestPrices'];
    portsItineraryCount?: number;
    recommendedArticles?: Array<Schema['MetaRelatedArticle']> | null;
    recommendedReviews?: Array<Schema['MetaRecommendedReview']> | null;
    relatedArticles?: Array<Schema['MetaRelatedArticleResults']> | null;
    relatedArticleFromText?: Array<Schema['MetaRelatedArticleResults']> | null;
    reviewsAggregateUrls?: Array<Schema['MetaSitemapUrl']>;
    sailings?: Schema['MetaSailings'];
    sailingsById?: Array<Schema['MetaSailing']>;
    sailingsByShipSlugAndDate?: Schema['MetaItineraryAndSailings'];
    searchReviews?: Array<Schema['MetaSearchReviewsResults']> | null;
    searchReviewsWithFilters?: Schema['MetaSearchReviewResponse'];
    shipSearch?: Schema['MetaShipSearchResult'];
    shipMaidenYear?: number | null;
    shipMaidenDate?: string | null;
    shipPrimaryImage?: Schema['MetaImage'] | null;
    shipsReviews?: Array<Schema['MetaSearchReviewsResults']>;
  };
  'MetaRecommendedReview': {
    __typename?: 'MetaRecommendedReview';
    id: number | null;
  };
  'MetaRelatedArticle': {
    __typename?: 'MetaRelatedArticle';
    id: number | null;
    title: string | null;
  };
  'MetaRelatedArticleResults': {
    __typename?: 'MetaRelatedArticleResults';
    relatedArticles?: Array<Schema['MetaRelatedArticle']> | null;
  };
  'MetaReview': {
    __typename?: 'MetaReview';
    id: string | null;
    title: string | null;
    snippet: string | null;
    body: string | null;
    rating: number | null;
    cruiseDate: string | null;
    cruiseLength: number | null;
    url: string | null;
    cruiseExperienceLevel: Schema['MetaCruiseExperienceLevel'] | null;
  };
  'MetaReviewBy': {
    __typename?: 'MetaReviewBy';
    id: string | null;
    username: string | null;
    totalPosts: number;
    avatarUrl: string;
  /**
   * User's age rounded down to the nearest decade (e.g. 54 -> 50)
   */
    age: number | null;
    totalHelpfulVotes: number | null;
    totalReviews: number | null;
  };
  'MetaSailing': {
    __typename?: 'MetaSailing';
    id: number;
    departureDate: string;
    meta?: Schema['MetaMeta'] | null;
    lowestPrice?: Schema['MetaLowestPriceWithVendor'] | null;
  };
  'MetaSailingCabinType': | 'inside'| 'outside'| 'balcony'| 'suite';
  'MetaSailingFields': | 'pricing'| 'default'| 'minimal';
  'MetaSailingFilterInput': {
    packageType: Array<Schema['MetaPackageType']> | null;
    departureDate: string | null;
    departureDateEnd: string | null;
    departureDateInterval: number | null;
    minPrice: number | null;
    maxPrice: number | null;
    cabinType: Schema['MetaCabinType'] | null;
    cabinTypeId: number | null;
  };
  'MetaSailings': {
    __typename?: 'MetaSailings';
    totalResults: number;
    results?: Array<Schema['MetaSailing']>;
  };
  'MetaSailingsFields': | 'pricing'| 'checkPrices'| 'deals';
  'MetaSchedule': {
    __typename?: 'MetaSchedule';
    totalResults: number;
    results?: Array<Schema['MetaDay']>;
  };
  'MetaSearchDeviceType': | 'MOBILE'| 'TABLET'| 'DESKTOP';
  'MetaSearchReviewFilterCabinType': {
    __typename?: 'MetaSearchReviewFilterCabinType';
    subject?: Schema['MetaCabinTypes'];
    totalEntries: number;
  };
  'MetaSearchReviewFilterCruiseLine': {
    __typename?: 'MetaSearchReviewFilterCruiseLine';
    subject?: Schema['MetaCruiseLines'];
    totalEntries: number;
    isPopular: boolean | null;
  };
  'MetaSearchReviewFilterCruiseStyle': {
    __typename?: 'MetaSearchReviewFilterCruiseStyle';
    subject?: Schema['MetaCruiseStyles'];
    totalEntries: number;
  };
  'MetaSearchReviewFilterDeparturePortPort': {
    __typename?: 'MetaSearchReviewFilterDeparturePortPort';
    subject?: Schema['MetaPorts'];
    totalEntries: number;
    isPopular: boolean | null;
  };
  'MetaSearchReviewFilterDestination': {
    __typename?: 'MetaSearchReviewFilterDestination';
    subject?: Schema['MetaDestinations'];
    totalEntries: number;
  };
  'MetaSearchReviewFilterRating': {
    __typename?: 'MetaSearchReviewFilterRating';
    subject?: Schema['MetaSearchReviewRating'];
    totalEntries: number;
  };
  'MetaSearchReviewFilterShip': {
    __typename?: 'MetaSearchReviewFilterShip';
    subject?: Schema['MetaShips'];
    totalEntries: number;
  };
  'MetaSearchReviewFilters': {
    __typename?: 'MetaSearchReviewFilters';
    cruiseLines?: Array<Schema['MetaSearchReviewFilterCruiseLine']> | null;
    ships?: Array<Schema['MetaSearchReviewFilterShip']> | null;
    destinations?: Array<Schema['MetaSearchReviewFilterDestination']> | null;
    departurePortPorts?: Array<Schema['MetaSearchReviewFilterDeparturePortPort']> | null;
    cruiseStyles?: Array<Schema['MetaSearchReviewFilterCruiseStyle']> | null;
    ratings?: Array<Schema['MetaSearchReviewFilterRating']> | null;
    cabinTypes?: Array<Schema['MetaSearchReviewFilterCabinType']> | null;
  };
  'MetaSearchReviewInput': {
    reviewId: Array<number> | null;
    cruiseLineId: Array<number> | null;
    shipId: Array<number> | null;
    destinationId: Array<number> | null;
    departurePortPortId: Array<number> | null;
    portId: Array<number> | null;
    cruiseStyleId: Array<number> | null;
    rating: Array<number> | null;
    subjectId: Array<number> | null;
    isPhotoJournal: boolean | null;
    cabinTypeId: Array<number> | null;
  };
  'MetaSearchReviewRating': {
    __typename?: 'MetaSearchReviewRating';
    id: number;
    name: string | null;
  };
  'MetaSearchReviewResponse': {
    __typename?: 'MetaSearchReviewResponse';
    totalResults: number;
    results?: Array<Schema['MetaSearchReviewsResults']>;
    filters?: Schema['MetaSearchReviewFilters'];
    stats?: Schema['MetaSearchReviewStats'];
    mostHelpfulReview?: Schema['MetaSearchReviewsResults'] | null;
  };
  'MetaSearchReviewStats': {
    __typename?: 'MetaSearchReviewStats';
    maxCruiseYear: number | null;
    maxPublishYear: number | null;
    averageMemberRating: number | null;
  };
  'MetaSearchReviewsResults': {
    __typename?: 'MetaSearchReviewsResults';
    review?: Schema['MetaReview'] | null;
    ship?: Schema['MetaShips'] | null;
    cruiseLine?: Schema['MetaCruiseLines'] | null;
    departurePort?: Schema['MetaPorts'] | null;
    destination?: Schema['MetaDestinations'] | null;
    helpfulVotes: number | null;
    totalImages: number | null;
    images?: Array<Schema['MetaSearchReviewsResultsImage']> | null;
    user?: Schema['MetaReviewBy'] | null;
  };
  'MetaSearchReviewsResultsImage': {
    __typename?: 'MetaSearchReviewsResultsImage';
    id: number;
    fileName: string | null;
  };
  'MetaShipInclusion': {
    __typename?: 'MetaShipInclusion';
    name: string;
    value: string;
    description: string | null;
  };
  'MetaShipInclusions': {
    __typename?: 'MetaShipInclusions';
    totalResults: number;
    results?: Array<Schema['MetaShipInclusion']>;
  };
  'MetaShipItineraryFilter': {
    __typename?: 'MetaShipItineraryFilter';
    id: string;
    name: string;
    totalResults: number;
    lowestPrice: number | null;
  };
  'MetaShipItineraryFilterSet': {
    __typename?: 'MetaShipItineraryFilterSet';
    totalResults: number;
    results?: Array<Schema['MetaShipItineraryFilter']>;
  };
  'MetaShipSearchResult': {
    __typename?: 'MetaShipSearchResult';
    results?: Array<Schema['MetaShips']>;
    totalResults: number;
    currentPage: number;
    totalPages: number;
    previousPage: number | null;
    nextPage: number | null;
  };
  'MetaShipSearchSortOrder': | 'NameAscending'| 'NameDescending'| 'DateLaunched'| 'Rating'| 'Price'| 'Popularity';
  'MetaShips': {
    __typename?: 'MetaShips';
    id: number;
    name: string;
    memberLovePercentage: number | null;
    maidenDate: string | null;
    maidenYear: number | null;
    averageMemberRating: number | null;
    professionalOverallRating: string | null;
    totalMemberReviews: number | null;
    imageUrl: string;
    primaryImage?: Schema['MetaImage'] | null;
    lowestPricePerNight: number | null;
    reviewsUrl: string | null;
    reviews?: Array<Schema['MetaSearchReviewsResults']>;
    seo?: Schema['DbSeo'] | null;
    mappedImage?: Schema['DbImages'] | null;
    image: string | null;
    mappedImages?: Array<Schema['DbImageMappings'] | null> | null;
    snippets?: Array<Schema['DbShipSnippets'] | null> | null;
    hasUserPhotos: boolean | null;
    hasItineraries: boolean | null;
    snippetsForTypes?: Array<Schema['DbShipSnippets'] | null> | null;
    attributes?: Schema['DbShipAttributes'] | null;
    ratio: string | null;
    amenitiesByType?: Schema['DbShipAmenityResponse'] | null;
    destinations?: Array<Schema['DbDestinations'] | null> | null;
    ports?: Array<Schema['DbPorts'] | null> | null;
    pastSailings?: Array<Schema['DbStoredSailings'] | null> | null;
    cruisersChoiceAwards?: Array<Schema['DbCruisersChoiceCategories'] | null> | null;
    cruisersChoiceDestinationAwards?: Array<Schema['DbCruisersChoiceCategories'] | null> | null;
    editorsPicksAwards?: Array<Schema['DbEditorsPicksCategories'] | null> | null;
    editorsPicksResults?: Array<Schema['DbEditorsPicksResults'] | null> | null;
    cruiseStyles?: Array<Schema['DbCruiseStyles'] | null> | null;
    totalShoreExcursions: number | null;
  };
  'MetaSitemapUrl': {
    __typename?: 'MetaSitemapUrl';
    url: string;
    lastModified: string;
  };
  'MetaSponsoredItinerariesResult': {
    __typename?: 'MetaSponsoredItinerariesResult';
    items?: Array<Schema['MetaItinerary']> | null;
  };
  'MetaStoredSailings': {
    __typename?: 'MetaStoredSailings';
    id: number;
    itineraryId: number;
    itinerary?: Schema['MetaItinerary'] | null;
  };
  'Meta_Any': any;
  'Meta_Entity': | Schema['MetaCabinTypes'] | Schema['MetaCruiseLineDeparturePort'] | Schema['MetaCruiseLineDestination'] | Schema['MetaCruiseLines'] | Schema['MetaCruiseLineShip'] | Schema['MetaCruiseStyles'] | Schema['MetaDeparturePorts'] | Schema['MetaDestinations'] | Schema['MetaItineraryPort'] | Schema['MetaItineraryShip'] | Schema['MetaPorts'] | Schema['MetaProviderDealBenefitTypes'] | Schema['MetaReview'] | Schema['MetaReviewBy'] | Schema['MetaShips'] | Schema['MetaStoredSailings'];
  'Meta_Service': {
    __typename?: 'Meta_Service';
  /**
   * The sdl representing the federated service capabilities. Includes federation directives, removes federation types, and includes rest of full schema after schema directives have been applied
   */
    sdl: string | null;
  };
  'Mutation': {
    __typename?: 'Mutation';
  /**
   * Access to embedded DB API.
   */
    db?: Schema['DbMutation'];
  /**
   * Access to embedded Reviews API.
   */
    reviews?: Schema['ReviewsMutation'];
  /**
   * Access to embedded Analytics API.
   */
    analytics?: Schema['AnalyticsMutation'];
  /**
   * Access to embedded Partners API.
   */
    partners?: Schema['PartnersMutation'];
  /**
   * Access to embedded Search API.
   */
    search?: Schema['SearchMutation'];
  /**
   * Access to embedded Personalization API.
   */
    personalization?: Schema['PersonalizationMutation'];
  };
  'PartnersAnalysedChatbotQuestion': {
    __typename?: 'PartnersAnalysedChatbotQuestion';
    question: string;
    intent: Schema['PartnersChatbotQuestionIntent'];
    cruiselines?: Array<Schema['PartnersCruiselineEntity']> | null;
    ships?: Array<Schema['PartnersShipEntity']> | null;
    cruisestyles?: Array<Schema['PartnersCruiseStyleEntity']> | null;
    durations?: Array<Schema['PartnersDurationEntity']> | null;
    dates?: Array<Schema['PartnersSailingDateEntity']> | null;
    destinations?: Array<Schema['PartnersDestinationOrPortEntity']> | null;
    sources?: Array<Schema['PartnersSourceEntity']> | null;
  };
  'PartnersAvailableDealFilterUnion': | Schema['PartnersCruiseStyleDealFilter'] | Schema['PartnersDestinationsDealFilter'] | Schema['PartnersLastMinuteDealFilter'] | Schema['PartnersCruiseLinesDealFilter'] | Schema['PartnersSailingYearDealFilter'] | Schema['PartnersDepartureMonthsDealFilter'] | Schema['PartnersDepartureCitiesDealFilter'] | Schema['PartnersLengthDealFilter'] | Schema['PartnersBenefitsDealFilter'] | Schema['PartnersCabinTypeDealFilter'];
  'PartnersAvailableDealFilters': {
    __typename?: 'PartnersAvailableDealFilters';
    lastMinute?: Array<Schema['PartnersDealFilterLastMinute']> | null;
    cruiseLines?: Array<Schema['PartnersDealFilterCruiseLines']> | null;
    year?: Array<Schema['PartnersDealFilterYear']> | null;
    departureMonths?: Array<Schema['PartnersDealFilterDepartureMonths']> | null;
    departureCities?: Array<Schema['PartnersDealFilterDepartureCities']> | null;
    destinations?: Array<Schema['PartnersDealFilterDestinations']> | null;
    styles?: Array<Schema['PartnersDealFilterStyles']> | null;
    length?: Array<Schema['PartnersDealFilterLength']> | null;
    season?: Array<Schema['PartnersDealFilterSeason']> | null;
    offers?: Array<Schema['PartnersDealFilterOffers']> | null;
    rooms?: Array<Schema['PartnersDealFilterRooms']> | null;
  };
  'PartnersBenefitsDealFilter': {
    __typename?: 'PartnersBenefitsDealFilter';
    id: string;
    count: number;
    filter: string;
    slug: string;
    type: string;
  };
  'PartnersBrowserInput': {
    userAgent: string;
    deviceType: string;
    browserFamily: string;
    devicePlatform: string;
    deviceOrientation: string;
  };
  'PartnersBudgets': {
    __typename?: 'PartnersBudgets';
    id: number;
    budget: number;
    totalClicks: number;
    currentSpend: string;
    startDate: string;
    cpc: number;
  };
  'PartnersCabinTypeDealFilter': {
    __typename?: 'PartnersCabinTypeDealFilter';
    id: string;
    count: number;
    filter: string;
    slug: string;
    type: string;
  };
  'PartnersChatbotMessageInput': {
    id: string;
    text: string;
    isBot: boolean;
    flagged: boolean;
    rating: Schema['PartnersChatbotMessageRating'] | null;
    articleIds: Array<number> | null;
    intention: Schema['PartnersChatbotQuestionIntent'] | null;
  };
  'PartnersChatbotMessageRating': | 'UP'| 'DOWN';
  'PartnersChatbotQuestionIntent': | 'CruiselineInfo'| 'CruisingKnowledge'| 'FindCruise'| 'Greet'| 'LocationInfo'| 'None'| 'Pricing'| 'ShipInfo'| 'SiteSupport';
  'PartnersChatbotSessionParameters': {
    browser: Schema['PartnersBrowserInput'];
    templateName: string;
    testVariations: string;
    tpixel: Array<Schema['PartnersTPixelInputParam']> | null;
  };
  'PartnersCountries': {
    __typename?: 'PartnersCountries';
    id: number;
    shortName: string;
  };
  'PartnersCruiseLinesDealFilter': {
    __typename?: 'PartnersCruiseLinesDealFilter';
    id: string;
    count: number;
    filter: string;
    slug: string;
    type: string;
  };
  'PartnersCruiseStyleDealFilter': {
    __typename?: 'PartnersCruiseStyleDealFilter';
    id: string;
    count: number;
    filter: string;
    slug: string;
    type: string;
  };
  'PartnersCruiseStyleEntity': {
    __typename?: 'PartnersCruiseStyleEntity';
    text: string;
    id: number | null;
  };
  'PartnersCruiselineEntity': {
    __typename?: 'PartnersCruiselineEntity';
    text: string;
    id: number | null;
  };
  /**
   * A date string, such as 2007-12-03, compliant with the `full-date` format outlined in section 5.6 of the RFC 3339 profile of the ISO 8601 standard for representation of dates and times using the Gregorian calendar.
   */
  'PartnersDate': any;
  'PartnersDateResolution': {
    __typename?: 'PartnersDateResolution';
    timex: string | null;
    begin: string | null;
    end: string | null;
    value: string | null;
  };
  /**
   * Cruise lines filter
   */
  'PartnersDealFilterCruiseLines': {
    __typename?: 'PartnersDealFilterCruiseLines';
    id: string;
    count: number;
    filter: string;
    slug: string;
  };
  /**
   * Departure cities filter
   */
  'PartnersDealFilterDepartureCities': {
    __typename?: 'PartnersDealFilterDepartureCities';
    id: string;
    count: number;
    filter: string;
    slug: string;
  };
  /**
   * Departure months filter
   */
  'PartnersDealFilterDepartureMonths': {
    __typename?: 'PartnersDealFilterDepartureMonths';
    id: string;
    count: number;
    filter: string;
    slug: string;
  };
  /**
   * Destinations filter
   */
  'PartnersDealFilterDestinations': {
    __typename?: 'PartnersDealFilterDestinations';
    id: string;
    count: number;
    filter: string;
    slug: string;
  };
  /**
   * Last minute deals filter
   */
  'PartnersDealFilterLastMinute': {
    __typename?: 'PartnersDealFilterLastMinute';
    id: string;
    count: number;
    filter: string;
    slug: string;
  };
  /**
   * Length filter
   */
  'PartnersDealFilterLength': {
    __typename?: 'PartnersDealFilterLength';
    id: string;
    count: number;
    filter: string;
    slug: string;
  };
  /**
   * Offers filter
   */
  'PartnersDealFilterOffers': {
    __typename?: 'PartnersDealFilterOffers';
    id: string;
    count: number;
    filter: string;
    slug: string;
  };
  /**
   * Rooms filter
   */
  'PartnersDealFilterRooms': {
    __typename?: 'PartnersDealFilterRooms';
    id: string;
    count: number;
    filter: string;
    slug: string;
  };
  /**
   * Season filter
   */
  'PartnersDealFilterSeason': {
    __typename?: 'PartnersDealFilterSeason';
    id: string;
    count: number;
    filter: string;
    slug: string;
  };
  /**
   * Styles filter
   */
  'PartnersDealFilterStyles': {
    __typename?: 'PartnersDealFilterStyles';
    id: string;
    count: number;
    filter: string;
    slug: string;
  };
  /**
   * Year filter
   */
  'PartnersDealFilterYear': {
    __typename?: 'PartnersDealFilterYear';
    id: string;
    count: number;
    filter: string;
    slug: string;
  };
  'PartnersDealScoreCalculation': {
    __typename?: 'PartnersDealScoreCalculation';
    score: number;
    formula: string | null;
    formulaWithNames: string | null;
    formulaAsHtml: string | null;
  };
  'PartnersDepartureCitiesDealFilter': {
    __typename?: 'PartnersDepartureCitiesDealFilter';
    id: string;
    count: number;
    filter: string;
    slug: string;
    type: string;
  };
  'PartnersDepartureMonthsDealFilter': {
    __typename?: 'PartnersDepartureMonthsDealFilter';
    id: string;
    count: number;
    filter: string;
    slug: string;
    type: string;
  };
  'PartnersDeparturePorts': {
    __typename?: 'PartnersDeparturePorts';
    id: number;
  };
  'PartnersDestinationOrPortEntity': {
    __typename?: 'PartnersDestinationOrPortEntity';
    text: string;
    portId: number | null;
    destinationId: number | null;
  };
  'PartnersDestinations': {
    __typename?: 'PartnersDestinations';
    id: number;
    ports?: Array<Schema['DbPorts'] | null> | null;
    image?: string | null;
    ships?: Array<Schema['DbShips'] | null> | null;
  };
  'PartnersDestinationsDealFilter': {
    __typename?: 'PartnersDestinationsDealFilter';
    id: string;
    count: number;
    filter: string;
    slug: string;
    type: string;
  };
  'PartnersDurationEntity': {
    __typename?: 'PartnersDurationEntity';
    text: string;
    resolutions?: Array<Schema['PartnersDurationResolution']>;
  };
  'PartnersDurationResolution': {
    __typename?: 'PartnersDurationResolution';
    timex: string | null;
    duration: string | null;
  };
  'PartnersEmailOnAcidCreateResponse': {
    __typename?: 'PartnersEmailOnAcidCreateResponse';
    success: boolean;
    message: string | null;
    id: string | null;
  };
  'PartnersEmailOnAcidTestResults': {
    __typename?: 'PartnersEmailOnAcidTestResults';
    display_name: string | null;
    client: string | null;
    os: string | null;
    category: string | null;
    browser: string | null;
    thumbnail: string | null;
    screenshots?: Schema['PartnersEmailOnAcidTestResultsScreenshots'];
    status: string | null;
  };
  'PartnersEmailOnAcidTestResultsScreenshots': {
    __typename?: 'PartnersEmailOnAcidTestResultsScreenshots';
    default: string | null;
  };
  'PartnersISnippet': {
    id: string;
    content: string;
  };
  'PartnersInviteProviderUserResult': {
    __typename?: 'PartnersInviteProviderUserResult';
    invited: boolean;
    inserted: boolean;
    exception: string | null;
    success: boolean;
  };
  /**
   * The `JSONObject` scalar type represents JSON objects as specified by [ECMA-404](http://www.ecma-international.org/publications/files/ECMA-ST/ECMA-404.pdf).
   */
  'PartnersJSONObject': any;
  'PartnersLastMinuteDealFilter': {
    __typename?: 'PartnersLastMinuteDealFilter';
    id: string;
    count: number;
    filter: string;
    slug: string;
    type: string;
  };
  'PartnersLengthDealFilter': {
    __typename?: 'PartnersLengthDealFilter';
    id: string;
    count: number;
    filter: string;
    slug: string;
    type: string;
  };
  'PartnersMKoCCabinPricing': {
    __typename?: 'PartnersMKoCCabinPricing';
    gradeNo: string;
    rateCode: string;
    infCode: string;
    price: number;
    perPersonPrice: number;
    title: string | null;
    description: string | null;
  };
  'PartnersMKoCCabinResult': {
    __typename?: 'PartnersMKoCCabinResult';
    name: string | null;
    imageUrl: string;
    minPrice: number | null;
    description: string;
    cabinCode: string | null;
    resultNo: string | null;
    pricing?: Array<Schema['PartnersMKoCCabinPricing']> | null;
  };
  'PartnersMKoCCabinResults': {
    __typename?: 'PartnersMKoCCabinResults';
    inside?: Schema['PartnersMKoCCabinResult'] | null;
    outside?: Schema['PartnersMKoCCabinResult'] | null;
    balcony?: Schema['PartnersMKoCCabinResult'] | null;
    suite?: Schema['PartnersMKoCCabinResult'] | null;
  };
  'PartnersMKoCCabinResultsByGrade': {
    __typename?: 'PartnersMKoCCabinResultsByGrade';
    inside?: Array<Schema['PartnersMKoCCabinResult']> | null;
    outside?: Array<Schema['PartnersMKoCCabinResult']> | null;
    balcony?: Array<Schema['PartnersMKoCCabinResult']> | null;
    suite?: Array<Schema['PartnersMKoCCabinResult']> | null;
    sessionKey: string | null;
    mkocSessionKey: string | null;
    adults: number | null;
    children: number | null;
    infants: number | null;
  };
  'PartnersMKoCShipDeck': {
    __typename?: 'PartnersMKoCShipDeck';
    name: string | null;
    deckCode: string | null;
    imageUrl: string | null;
    staterooms?: Array<Schema['PartnersMKoCStateroom']>;
    id: number | null;
  };
  'PartnersMKoCStateroom': {
    __typename?: 'PartnersMKoCStateroom';
    cabinResultNo: string;
    cabinNo: string;
    maxGuests: number | null;
    minGuests: number | null;
    coordinates?: Schema['PartnersMKoCStateroomCoordinates'] | null;
  };
  'PartnersMKoCStateroomCoordinates': {
    __typename?: 'PartnersMKoCStateroomCoordinates';
    x1: number;
    x2: number;
    y1: number;
    y2: number;
  };
  'PartnersMKoCStateroomResults': {
    __typename?: 'PartnersMKoCStateroomResults';
    decks?: Array<Schema['PartnersMKoCShipDeck']>;
    guaranteed?: Schema['PartnersMKoCStateroom'] | null;
  };
  'PartnersMutation': {
    __typename?: 'PartnersMutation';
    renderSponsoredEmail?: Schema['PartnersSponsoredEmailResponse'];
    startChatbotSession?: boolean;
    createChatbotMessage?: boolean;
    endChatbotSession?: boolean;
    rateChatbotMessage?: boolean;
    flagChatbotMessage?: boolean;
    addChatbotMessageIntent?: boolean;
    updateProviderDealBenefit?: Schema['PartnersProviderDealBenefits'];
    createProviderDeal?: Schema['PartnersProviderDeals'];
    updateProviderDeal?: Schema['PartnersProviderDeals'];
    trackProviderDealClick?: boolean;
    trackConversion?: boolean;
    updateProviderEmailDraft?: Schema['PartnersProviderEmailDrafts'];
    createProviderEmailDraft?: Schema['PartnersProviderEmailDrafts'];
    createEmailOnAcidTest?: Schema['PartnersEmailOnAcidCreateResponse'] | null;
    updateProviderEmail?: Schema['PartnersProviderEmails'];
    createProviderEmail?: Schema['PartnersProviderEmails'];
    updateProvider?: Schema['PartnersProviders'];
    inviteProviderUser?: Schema['PartnersInviteProviderUserResult'];
    removeProviderUser?: Schema['PartnersRemoveProviderUserResult'];
    resendProviderUser?: Schema['PartnersInviteProviderUserResult'];
    setProviderUserRoles?: Schema['PartnersProviderRoleResult'];
    triggerSponsoredEmailTestSend?: Schema['PartnersSponsoredEmailResponse'];
    submitSponsoredEmailToCruiseCritic?: Schema['PartnersSponsoredEmailResponse'];
    sendSponsoredEmailWelcomeEmail?: Schema['PartnersSponsoredEmailResponse'];
    sendSponsoredEmailApprovedEmail?: Schema['PartnersSponsoredEmailResponse'];
    sendSponsoredEmailRequestEditEmail?: Schema['PartnersSponsoredEmailResponse'];
  };
  'PartnersPaginatedProviderDealBenefits': {
    __typename?: 'PartnersPaginatedProviderDealBenefits';
    results?: Array<Schema['PartnersProviderDealBenefits']>;
    totalResults: number;
    currentPage: number;
    totalPages: number;
    previousPage: number | null;
    nextPage: number | null;
    totalValue: number;
  };
  'PartnersPaginatedProviderDeals': {
    __typename?: 'PartnersPaginatedProviderDeals';
    results?: Array<Schema['PartnersProviderDeals']>;
    totalResults: number;
    currentPage: number;
    totalPages: number;
    previousPage: number | null;
    nextPage: number | null;
    filters?: Array<Schema['PartnersAvailableDealFilterUnion']>;
  };
  'PartnersPaginatedProviderEmails': {
    __typename?: 'PartnersPaginatedProviderEmails';
    results?: Array<Schema['PartnersProviderEmails']>;
    totalResults: number;
    currentPage: number;
    totalPages: number;
    previousPage: number | null;
    nextPage: number | null;
  };
  'PartnersPaginatedProviders': {
    __typename?: 'PartnersPaginatedProviders';
    results?: Array<Schema['PartnersProviders']>;
    totalResults: number;
    currentPage: number;
    totalPages: number;
    previousPage: number | null;
    nextPage: number | null;
  };
  'PartnersProviderBenefitOfferType': | 'Offer'| 'Inclusion';
  'PartnersProviderBenefitOfferValueValidationType': | 'GreaterThan'| 'LessThan';
  'PartnersProviderDealActivityStatus': | 'PendingReview'| 'Active'| 'Inactive'| 'Rejected'| 'Paused'| 'Archived';
  'PartnersProviderDealApprovalStatus': | 'New'| 'Approved'| 'Rejected';
  'PartnersProviderDealBenefitInput': {
    id: number | null;
    description: string;
    value: number | null;
    isAlwaysIncluded: boolean | null;
    providerDealBenefitTypeId: string;
  };
  'PartnersProviderDealBenefitTypeSort': | 'popularity'| 'description';
  'PartnersProviderDealBenefitTypes': {
    __typename?: 'PartnersProviderDealBenefitTypes';
    id: string;
    description: string;
    weight: number;
    hasValue: boolean;
    valuePlaceholder: string;
    valueTooltip: string;
    valueTooltipFailure: string;
    valueValidationType: Schema['PartnersProviderBenefitOfferValueValidationType'] | null;
    valueValidation: string;
    offerType: Schema['PartnersProviderBenefitOfferType'];
    slug: string | null;
  };
  'PartnersProviderDealBenefitUpdateInput': {
    description: string | null;
    value: number | null;
    providerDealBenefitTypeId: string | null;
  };
  'PartnersProviderDealBenefits': {
    __typename?: 'PartnersProviderDealBenefits';
    id: number;
    providerDealBenefitTypeId: string;
    description: string;
    value: number | null;
    isAlwaysIncluded: boolean | null;
    type?: Schema['PartnersProviderDealBenefitTypes'] | null;
  };
  'PartnersProviderDealClickParameters': {
    transactionId: string | null;
    url: string;
    source: number | null;
    browser: Schema['PartnersBrowserInput'];
    eventUrl: string;
    templateName: string;
    position: number;
    score: number;
    testVariations: string;
    tpixel: Array<Schema['PartnersTPixelInputParam']> | null;
  };
  'PartnersProviderDealCreateInput': {
    name: string;
    title: string;
    imageUrl: string | null;
    startDate: string;
    endDate: string;
    isExclusive: boolean;
    isVacationPackage: boolean;
    isSoloSupplementalDeal: boolean;
    shipId: number;
    destinationId: number;
    departurePortId: number;
    length: number;
    sailingDateType: Schema['PartnersProviderSailingDateType'];
    sailingStartDate: string | null;
    sailingEndDate: string | null;
    originalPrice: number;
    price: number;
    url: string | null;
    cabinType: Schema['PartnersRealCabinType'] | null;
    benefits: Array<Schema['PartnersProviderDealBenefitInput']>;
  };
  'PartnersProviderDealLength': | 'oneToTwo'| 'threeToFive'| 'sixToNine'| 'tenToFourteen'| 'fifteenPlus'| 'weekend';
  'PartnersProviderDealOrder': | 'id'| 'score'| 'recent'| 'random'| 'cheapest'| 'clickThroughRate';
  'PartnersProviderDealRejectReasonInput': {
    providerDealRejectionTypeId: number;
    note: string | null;
  };
  'PartnersProviderDealRejectionReasons': {
    __typename?: 'PartnersProviderDealRejectionReasons';
    id: number;
    providerDealId: number;
    providerDealRejectionTypeId: number;
    note: string | null;
    rejectionType?: Schema['PartnersProviderDealRejectionTypes'] | null;
    reason: string;
  };
  'PartnersProviderDealRejectionTypes': {
    __typename?: 'PartnersProviderDealRejectionTypes';
    id: number;
    description: string;
  };
  'PartnersProviderDealSeason': | 'winter'| 'spring'| 'summer'| 'fall';
  'PartnersProviderDealUpdateInput': {
    name: string | null;
    title: string | null;
    imageUrl: string | null;
    startDate: string | null;
    endDate: string | null;
    isExclusive: boolean | null;
    isVacationPackage: boolean | null;
    isSoloSupplementalDeal: boolean | null;
    shipId: number | null;
    destinationId: number | null;
    departurePortId: number | null;
    length: number | null;
    sailingDateType: Schema['PartnersProviderSailingDateType'] | null;
    sailingStartDate: string | null;
    sailingEndDate: string | null;
    price: number | null;
    originalPrice: number | null;
    url: string | null;
    cabinType: Schema['PartnersRealCabinType'] | null;
    benefits: Array<Schema['PartnersProviderDealBenefitInput']> | null;
    approvalStatus: Schema['PartnersProviderDealApprovalStatus'] | null;
    isEnabled: boolean | null;
    isArchived: boolean | null;
    rejectionReasons: Array<Schema['PartnersProviderDealRejectReasonInput']> | null;
  };
  'PartnersProviderDeals': {
    __typename?: 'PartnersProviderDeals';
    id: number;
    providerId: number;
    name: string;
    title: string;
    featuredDealText: string | null;
    imageUrl: string | null;
    startDate: Schema['PartnersDate'];
    endDate: Schema['PartnersDate'];
    isExclusive: boolean;
    isVacationPackage: boolean;
    isSoloSupplementalDeal: boolean;
    shipId: number;
    destinationId: number;
    departurePortId: number | null;
    length: number;
    sailingDateType: Schema['PartnersProviderSailingDateType'];
    sailingStartDate: Schema['PartnersDate'] | null;
    sailingEndDate: Schema['PartnersDate'] | null;
    originalPrice: number;
    price: number;
    score: number | null;
    url: string | null;
    urlWithAppend: string | null;
    cabinType: Schema['PartnersRealCabinType'] | null;
    approvalStatus: Schema['PartnersProviderDealApprovalStatus'];
    activityStatus: Schema['PartnersProviderDealActivityStatus'] | null;
    isEnabled: boolean;
    isArchived: boolean;
    totalImpressions: number;
    totalDimensions: number;
    createdAt: Schema['PartnersDate'];
    benefits?: Array<Schema['PartnersProviderDealBenefits']>;
    paginatedBenefits?: Schema['PartnersPaginatedProviderDealBenefits'];
    ship?: Schema['DbShips'] | null;
    destination?: Schema['DbDestinations'] | null;
    departurePort?: Schema['DbDeparturePorts'] | null;
    provider?: Schema['PartnersProviders'] | null;
    rank?: number | null;
    rejectionReasons?: Array<Schema['PartnersProviderDealRejectionReasons']> | null;
    countryId: number | null;
    cpc: string | null;
    itineraries?: Schema['SearchProviderDealItinerariesResult'] | null;
  };
  'PartnersProviderEmailCountry': {
    __typename?: 'PartnersProviderEmailCountry';
    id: number;
    name: string | null;
    short_name: string | null;
  };
  'PartnersProviderEmailCreateInput': {
    projectName: string;
    clientEmailAddresses: string;
    providerEmailTemplateId: number;
    providerId: number;
    sendOn: Schema['PartnersDate'];
    dueOn: Schema['PartnersDate'];
    status: string | null;
    shouldSendReminderEmails: boolean | null;
    totalTestSends: number | null;
    maximumTestSends: number | null;
    isArchived: boolean | null;
    sendWelcomeEmail: boolean | null;
  };
  'PartnersProviderEmailDraftCreateInput': {
    providerEmailId: string;
    subject: string;
    preHeader: string;
    body: string;
    emailOnAcidId: string | null;
    feedback: string | null;
    isActive: boolean | null;
  };
  'PartnersProviderEmailDraftUpdateInput': {
    subject: string | null;
    preHeader: string | null;
    body: string | null;
    emailOnAcidId: string | null;
    feedback: string | null;
    isActive: boolean | null;
  };
  'PartnersProviderEmailDrafts': {
    __typename?: 'PartnersProviderEmailDrafts';
    id: number;
    providerEmailId: string;
    subject: string;
    preHeader: string;
    body: string;
    emailOnAcidId: string | null;
    feedback: string | null;
    isActive: boolean;
    previews?: Array<Schema['PartnersEmailOnAcidTestResults']> | null;
  };
  'PartnersProviderEmailFilterInput': {
    projectName: string | null;
    clientEmailAddresses: string | null;
    providerEmailTemplateId: number | null;
    providerId: number | null;
    sendOn: Schema['PartnersDate'] | null;
    dueOn: Schema['PartnersDate'] | null;
    status: string | null;
    shouldSendReminderEmails: boolean | null;
    totalTestSends: number | null;
    maximumTestSends: number | null;
    totalPreviews: number | null;
    maximumPreviews: number | null;
    isArchived: boolean | null;
    id: string | null;
  };
  'PartnersProviderEmailProvider': {
    __typename?: 'PartnersProviderEmailProvider';
    id: number;
    name: string;
  };
  'PartnersProviderEmailTemplates': {
    __typename?: 'PartnersProviderEmailTemplates';
    id: number;
    countryId: number;
    productId: number;
    name: string;
    exactTargetSendDefinitionKey: string;
  };
  'PartnersProviderEmailUpdateInput': {
    projectName: string | null;
    clientEmailAddresses: string | null;
    providerEmailTemplateId: number | null;
    providerId: number | null;
    sendOn: Schema['PartnersDate'] | null;
    dueOn: Schema['PartnersDate'] | null;
    status: string | null;
    shouldSendReminderEmails: boolean | null;
    totalTestSends: number | null;
    maximumTestSends: number | null;
    totalPreviews: number | null;
    maximumPreviews: number | null;
    isArchived: boolean | null;
  };
  'PartnersProviderEmails': {
    __typename?: 'PartnersProviderEmails';
    id: string;
    projectName: string;
    providerId: number;
    clientEmailAddresses: string;
    providerEmailTemplateId: number;
    shouldSendReminderEmails: boolean;
    sendOn: Schema['PartnersDate'];
    dueOn: Schema['PartnersDate'];
    status: string;
    totalTestSends: number;
    maximumTestSends: number;
    totalPreviews: number;
    maximumPreviews: number;
    isArchived: boolean;
    activeDraft?: Schema['PartnersProviderEmailDrafts'] | null;
    drafts?: Array<Schema['PartnersProviderEmailDrafts']> | null;
    template?: Schema['PartnersProviderEmailTemplates'] | null;
    country?: Schema['PartnersProviderEmailCountry'] | null;
    provider?: Schema['PartnersProviderEmailProvider'] | null;
  };
  'PartnersProviderParameters': {
    __typename?: 'PartnersProviderParameters';
    webAddress: string | null;
    phone: string | null;
    bio: string | null;
  };
  'PartnersProviderParametersInput': {
    webAddress: string | null;
    phone: string | null;
    bio: string | null;
  };
  'PartnersProviderPerformanceGoalInput': {
    providerPerformanceGoalTypeId: number;
    note: string | null;
  };
  'PartnersProviderPerformanceGoalTypes': {
    __typename?: 'PartnersProviderPerformanceGoalTypes';
    id: number;
    description: string;
    weight: number;
  };
  'PartnersProviderPerformanceGoals': {
    __typename?: 'PartnersProviderPerformanceGoals';
    id: number;
    providerId: number;
    providerPerformanceGoalTypeId: number;
    note: string | null;
    performanceGoalType?: Schema['PartnersProviderPerformanceGoalTypes'] | null;
    performanceGoal: string;
  };
  'PartnersProviderRole': | 'DealsPartner'| 'EmailPartner'| 'DealsAdmin';
  'PartnersProviderRoleResult': {
    __typename?: 'PartnersProviderRoleResult';
    success: boolean;
    message: string | null;
  };
  'PartnersProviderSailingDateType': | 'simple'| 'range';
  'PartnersProviderUserResult': {
    __typename?: 'PartnersProviderUserResult';
    providerId: number;
    email: string;
    userName: string;
    roles: Array<Schema['PartnersProviderRole']>;
  };
  'PartnersProviders': {
    __typename?: 'PartnersProviders';
    id: number;
    name: string;
    countryId: number;
    logoUrl: string | null;
    providesDeals: boolean;
    dealReportEmailAddresses: Array<string> | null;
    salesforceVendorId: string;
    joinedAt: Schema['PartnersDate'] | null;
    parameters?: Schema['PartnersProviderParameters'] | null;
    totalImpressions: number;
    totalDimensions: number;
    avgDealScore: number;
    usersEmail: Array<string> | null;
    performanceGoals?: Array<Schema['PartnersProviderPerformanceGoals']> | null;
    country?: Schema['PartnersCountries'] | null;
  };
  'PartnersProvidersUpdateInput': {
    name: string | null;
    logoUrl: string | null;
    providesDeals: boolean | null;
    dealReportEmailAddresses: Array<string> | null;
    parameters: Schema['PartnersProviderParametersInput'] | null;
    performanceGoals: Array<Schema['PartnersProviderPerformanceGoalInput']> | null;
  };
  'PartnersQuery': {
    __typename?: 'PartnersQuery';
    _entities?: Array<Schema['Partners_Entity'] | null>;
    _service?: Schema['Partners_Service'];
    analyzeChatbotMessage?: Schema['PartnersAnalysedChatbotQuestion'];
    subjectsSearchTermRelatedTexts?: Array<Schema['PartnersISnippet']> | null;
    mkocAvailableCabins?: Schema['PartnersMKoCCabinResults'] | null;
    mkocCabinsByShip?: Schema['PartnersMKoCCabinResults'] | null;
    mkocAvailableCabinsByGrade?: Schema['PartnersMKoCCabinResultsByGrade'] | null;
    mkocSelectCabin?: Schema['PartnersStatusMessageResponse'];
    mkocStateroomSelection?: Schema['PartnersMKoCStateroomResults'] | null;
    providerDealBenefitTypes?: Array<Schema['PartnersProviderDealBenefitTypes']>;
    providerDealBenefitTypeBySlug?: Array<Schema['PartnersProviderDealBenefitTypes']> | null;
    providerDealRejectionTypes?: Array<Schema['PartnersProviderDealRejectionTypes']>;
    providerPerformanceGoalTypes?: Array<Schema['PartnersProviderPerformanceGoalTypes']>;
    paginatedProviderDeals?: Schema['PartnersPaginatedProviderDeals'];
    providerDeal?: Schema['PartnersProviderDeals'] | null;
    budget?: Schema['PartnersBudgets'] | null;
    providerDeals?: Array<Schema['PartnersProviderDeals']>;
    dealScore?: number;
    dealScoreWithFormula?: Schema['PartnersDealScoreCalculation'];
    providerDealsFilters?: Array<Schema['PartnersAvailableDealFilterUnion']>;
    providerDealsFiltersV2?: Schema['PartnersAvailableDealFilters'];
    availableProviderDeal?: Schema['PartnersProviderDeals'] | null;
    availableProviderDeals?: Array<Schema['PartnersProviderDeals']>;
    paginatedAvailableProviderDeals?: Schema['PartnersPaginatedProviderDeals'];
    partialMatchedProviderDeals?: Array<Schema['PartnersProviderDeals']>;
    providerEmailDraft?: Schema['PartnersProviderEmailDrafts'] | null;
    getResultsFromEmailOnAcid?: Array<Schema['PartnersEmailOnAcidTestResults']>;
    providerEmail?: Schema['PartnersProviderEmails'] | null;
    providerEmails?: Array<Schema['PartnersProviderEmails']>;
    paginatedProviderEmails?: Schema['PartnersPaginatedProviderEmails'];
    providerEmailTemplate?: Schema['PartnersProviderEmailTemplates'] | null;
    providerEmailTemplates?: Array<Schema['PartnersProviderEmailTemplates']>;
    paginatedProviders?: Schema['PartnersPaginatedProviders'];
    provider?: Schema['PartnersProviders'] | null;
    providers?: Array<Schema['PartnersProviders']>;
    getProviderUsers?: Array<Schema['PartnersProviderUserResult']>;
  };
  'PartnersRealCabinType': | 'Inside'| 'Outside'| 'Balcony'| 'Suite';
  'PartnersRemoveProviderUserResult': {
    __typename?: 'PartnersRemoveProviderUserResult';
    removed: boolean;
    deleted: boolean;
    exception: string | null;
    success: boolean;
  };
  'PartnersSailingDateEntity': {
    __typename?: 'PartnersSailingDateEntity';
    text: string;
    resolutions?: Array<Schema['PartnersDateResolution']>;
  };
  'PartnersSailingYearDealFilter': {
    __typename?: 'PartnersSailingYearDealFilter';
    id: string;
    count: number;
    filter: string;
    slug: string;
    type: string;
  };
  'PartnersShipEntity': {
    __typename?: 'PartnersShipEntity';
    text: string;
    id: number | null;
  };
  'PartnersShipRelatedText': {
    __typename?: 'PartnersShipRelatedText';
    id: string;
    content: string;
    section: string;
    snippetName: string;
  };
  'PartnersShips': {
    __typename?: 'PartnersShips';
    id: number;
    maidenDate: string | null;
    maidenYear: number | null;
    primaryImage?: Schema['MetaImage'] | null;
    seo?: Schema['DbSeo'] | null;
    mappedImage?: Schema['DbImages'] | null;
    image: string | null;
    mappedImages?: Array<Schema['DbImageMappings'] | null> | null;
    snippets?: Array<Schema['DbShipSnippets'] | null> | null;
    hasUserPhotos: boolean | null;
    hasItineraries: boolean | null;
    snippetsForTypes?: Array<Schema['DbShipSnippets'] | null> | null;
    attributes?: Schema['DbShipAttributes'] | null;
    ratio: string | null;
    amenitiesByType?: Schema['DbShipAmenityResponse'] | null;
    destinations?: Array<Schema['DbDestinations'] | null> | null;
    ports?: Array<Schema['DbPorts'] | null> | null;
    pastSailings?: Array<Schema['DbStoredSailings'] | null> | null;
    cruisersChoiceAwards?: Array<Schema['DbCruisersChoiceCategories'] | null> | null;
    cruisersChoiceDestinationAwards?: Array<Schema['DbCruisersChoiceCategories'] | null> | null;
    editorsPicksAwards?: Array<Schema['DbEditorsPicksCategories'] | null> | null;
    editorsPicksResults?: Array<Schema['DbEditorsPicksResults'] | null> | null;
    cruiseStyles?: Array<Schema['DbCruiseStyles'] | null> | null;
    totalShoreExcursions: number | null;
  };
  'PartnersSortProviderDealsBy': {
    field: string;
    order: string;
  };
  'PartnersSourceEntity': {
    __typename?: 'PartnersSourceEntity';
    text: string;
    departurePortId: number | null;
  };
  'PartnersSponsoredEmailResponse': {
    __typename?: 'PartnersSponsoredEmailResponse';
    success: boolean;
    message: string | null;
  };
  'PartnersSponsoredEmailTestSendInput': {
    providerEmailId: string;
    subject: string;
    preHeader: string;
    body: string;
  };
  'PartnersStatusMessageResponse': {
    __typename?: 'PartnersStatusMessageResponse';
    message: string | null;
    success: boolean;
  };
  'PartnersSubjectId': | 'destinations'| 'ports'| 'ship'| 'cruiseline';
  'PartnersSubjectRelatedText': {
    __typename?: 'PartnersSubjectRelatedText';
    id: string;
    content: string;
    question: string | null;
    heading: string | null;
    answer: string;
    snippetType: string;
  };
  'PartnersTPixelInputParam': {
    key: string;
    value: string;
  };
  'Partners_Any': any;
  'Partners_Entity': | Schema['PartnersCountries'] | Schema['PartnersDeparturePorts'] | Schema['PartnersDestinations'] | Schema['PartnersProviderDealBenefitTypes'] | Schema['PartnersProviderDeals'] | Schema['PartnersProviders'] | Schema['PartnersShips'];
  'Partners_Service': {
    __typename?: 'Partners_Service';
  /**
   * The sdl representing the federated service capabilities. Includes federation directives, removes federation types, and includes rest of full schema after schema directives have been applied
   */
    sdl: string | null;
  };
  'PersonalizationAttribute': {
    __typename?: 'PersonalizationAttribute';
    key: string | null;
    value: Array<string> | null;
  };
  'PersonalizationAttributeInput': {
    key: string | null;
    value: Array<string> | null;
  };
  'PersonalizationEndpointDemographic': {
    __typename?: 'PersonalizationEndpointDemographic';
    appVersion: string | null;
    locale: string | null;
    make: string | null;
    model: string | null;
    modelVersion: string | null;
    platform: string | null;
    platformVersion: string | null;
    timezone: string | null;
  };
  'PersonalizationEndpointDemographicInput': {
    appVersion: string | null;
    locale: string | null;
    make: string | null;
    model: string | null;
    modelVersion: string | null;
    platform: string | null;
    platformVersion: string | null;
    timezone: string | null;
  };
  'PersonalizationEndpointLocation': {
    __typename?: 'PersonalizationEndpointLocation';
    city: string | null;
    country: string | null;
    latitude: number | null;
    longitude: number | null;
    postalCode: string | null;
    region: string | null;
  };
  'PersonalizationEndpointLocationInput': {
    city: string | null;
    country: string | null;
    latitude: number | null;
    longitude: number | null;
    postalCode: string | null;
    region: string | null;
  };
  'PersonalizationEndpointUser': {
    __typename?: 'PersonalizationEndpointUser';
    userAttributes?: Array<Schema['PersonalizationAttribute']> | null;
    userId: string | null;
  };
  'PersonalizationEndpointUserInput': {
    userAttributes: Array<Schema['PersonalizationAttributeInput']> | null;
    userId: string | null;
  };
  'PersonalizationMetrics': {
    __typename?: 'PersonalizationMetrics';
    key: string | null;
    value: number | null;
  };
  'PersonalizationMetricsInput': {
    key: string | null;
    value: number | null;
  };
  'PersonalizationMutation': {
    __typename?: 'PersonalizationMutation';
    updatePinpointEndpoint?: boolean;
    putPinpointEvents?: boolean;
    putPersonalizeEventsItem?: boolean;
  };
  'PersonalizationPersonalizeFilterValue': {
    key: string | null;
    value: string | null;
  };
  'PersonalizationPersonalizeFilterValues': {
    filters: Array<Schema['PersonalizationPersonalizeFilterValue']> | null;
  };
  'PersonalizationPersonalizeItem': {
    __typename?: 'PersonalizationPersonalizeItem';
    itemId: string;
    score: number | null;
  };
  'PersonalizationPersonalizeRecommendationResponse': {
    __typename?: 'PersonalizationPersonalizeRecommendationResponse';
    itemList?: Array<Schema['PersonalizationPersonalizeItem']> | null;
    recommendationId: string | null;
  };
  'PersonalizationPinpointEndpointRequest': {
    endpointId: string;
    address: string | null;
    attributes: Array<Schema['PersonalizationAttributeInput']> | null;
    channelType: string | null;
    demographic: Schema['PersonalizationEndpointDemographicInput'] | null;
    creationDate: string | null;
    effectiveDate: string | null;
    location: Schema['PersonalizationEndpointLocationInput'] | null;
    metrics: Array<Schema['PersonalizationMetricsInput']> | null;
    optOut: string | null;
    requestId: string | null;
    user: Schema['PersonalizationEndpointUserInput'] | null;
    encryptedAddress: string | null;
    etrid: number | null;
  };
  'PersonalizationPinpointEndpointResponse': {
    __typename?: 'PersonalizationPinpointEndpointResponse';
    address: string | null;
    applicationId: string | null;
    attributes?: Array<Schema['PersonalizationAttribute']> | null;
    channelType: string | null;
    cohortId: string | null;
    creationDate: string | null;
    demographic?: Schema['PersonalizationEndpointDemographic'] | null;
    effectiveDate: string | null;
    endpointStatus: string | null;
    id: string | null;
    location?: Schema['PersonalizationEndpointLocation'] | null;
    metrics?: Array<Schema['PersonalizationMetrics']> | null;
    optOut: string | null;
    requestId: string | null;
    user?: Schema['PersonalizationEndpointUser'] | null;
    etrid: number | null;
  };
  'PersonalizationPinpointPutEventsRequest': {
    endpointRequest: Schema['PersonalizationPinpointEndpointRequest'] | null;
    events: Array<Schema['PersonalizationPutEventsEventMap']> | null;
  };
  'PersonalizationPutEventsAttributeInput': {
    key: string | null;
    value: Array<string> | null;
  };
  'PersonalizationPutEventsEvent': {
    attributes: Array<Schema['PersonalizationPutEventsAttributeInput']> | null;
    eventType: string | null;
    pageType: string | null;
    metrics: Array<Schema['PersonalizationPutEventsMetricsInput']> | null;
    timestamp: string | null;
  };
  'PersonalizationPutEventsEventMap': {
    key: string | null;
    value: Schema['PersonalizationPutEventsEvent'] | null;
  };
  'PersonalizationPutEventsMetricsInput': {
    key: string | null;
    value: number | null;
  };
  'PersonalizationQuery': {
    __typename?: 'PersonalizationQuery';
    _service?: Schema['Personalization_Service'];
    getPinpointEndpoint?: Schema['PersonalizationPinpointEndpointResponse'];
    getPinpointUserEndpoints?: Array<Schema['PersonalizationPinpointEndpointResponse']>;
    getPersonalizeUserRecommendations?: Schema['PersonalizationPersonalizeRecommendationResponse'];
    getPersonalizePopularRecommendations?: Schema['PersonalizationPersonalizeRecommendationResponse'];
    getPersonalizeSimilarRecommendations?: Schema['PersonalizationPersonalizeRecommendationResponse'];
    getPersonalizePersonalizedRanking?: Schema['PersonalizationPersonalizeRecommendationResponse'];
  };
  'Personalization_Service': {
    __typename?: 'Personalization_Service';
  /**
   * The sdl representing the federated service capabilities. Includes federation directives, removes federation types, and includes rest of full schema after schema directives have been applied
   */
    sdl: string | null;
  };
  'Query': {
    __typename?: 'Query';
  /**
   * Access to embedded Storyblok API.
   */
    storyblok?: Schema['StoryblokQueryType'];
  /**
   * Access to embedded StoryblokDraft API.
   */
    storyblokDraft?: Schema['StoryblokDraftQueryType'];
  /**
   * Access to embedded DB API.
   */
    db?: Schema['DbQuery'];
  /**
   * Access to embedded SEO API.
   */
    seo?: Schema['SeoQuery'];
  /**
   * Access to embedded Reviews API.
   */
    reviews?: Schema['ReviewsQuery'];
  /**
   * Access to embedded Analytics API.
   */
    analytics?: Schema['AnalyticsQuery'];
  /**
   * Access to embedded Partners API.
   */
    partners?: Schema['PartnersQuery'];
  /**
   * Access to embedded Meta API.
   */
    meta?: Schema['MetaQuery'];
  /**
   * Access to embedded Search API.
   */
    search?: Schema['SearchQuery'];
  /**
   * Access to embedded Personalization API.
   */
    personalization?: Schema['PersonalizationQuery'];
    hello?: string;
  };
  /**
   * A date string, such as 2007-12-03, compliant with the `full-date` format outlined in section 5.6 of the RFC 3339 profile of the ISO 8601 standard for representation of dates and times using the Gregorian calendar.
   */
  'ReviewsDate': any;
  /**
   * A date-time string at UTC, such as 2007-12-03T10:15:30Z, compliant with the `date-time` format outlined in section 5.6 of the RFC 3339 profile of the ISO 8601 standard for representation of dates and times using the Gregorian calendar.
   */
  'ReviewsDateTime': any;
  'ReviewsDestinations': {
    __typename?: 'ReviewsDestinations';
    id: number;
    ports?: Array<Schema['DbPorts'] | null> | null;
    image?: string | null;
    ships?: Array<Schema['DbShips'] | null> | null;
  };
  'ReviewsMraUrl': {
    __typename?: 'ReviewsMraUrl';
    name: string;
    url: string | null;
  };
  'ReviewsMraUrlInput': {
    cruiseLineId: number | null;
    shipId: number | null;
    portId: number | null;
    destinationId: number | null;
    cruiseStyleId: number | null;
    rating: string | null;
    cabinTypeId: number | null;
  };
  'ReviewsMraUrlInputs': {
    name: string;
    filters: Schema['ReviewsMraUrlInput'];
  };
  'ReviewsMutation': {
    __typename?: 'ReviewsMutation';
    addHelpfulVote?: boolean;
    indexReview?: boolean;
    updateReviewsMapping: boolean;
    createReview?: Schema['ReviewsReviews'];
    addReviewEntries?: boolean;
    updateReview?: Schema['ReviewsReviews'];
  };
  'ReviewsQuery': {
    __typename?: 'ReviewsQuery';
    _entities?: Array<Schema['Reviews_Entity'] | null>;
    _service?: Schema['Reviews_Service'];
    reviewSlugBySlug?: Schema['ReviewsReviewSlugs'] | null;
    mraUrl?: string | null;
    mraUrls?: Array<Schema['ReviewsMraUrl']> | null;
    review?: Schema['ReviewsReviews'] | null;
    reviews?: Array<Schema['ReviewsReviews']>;
    reviewCountByUser?: number;
    userReviews?: Array<Schema['ReviewsReviews']>;
    helpfulVotesByUser?: number;
    shipsCategoryRating?: number | null;
  };
  'ReviewsReviewCabinPivotInput': {
    cabinCategory: string;
    cabinNumber: string | null;
  };
  'ReviewsReviewCabinPivots': {
    __typename?: 'ReviewsReviewCabinPivots';
    id: number;
    reviewEntryId: number;
    cabinCategory: string | null;
    cabinNumber: string | null;
  };
  'ReviewsReviewCategory': | 'enrichmentActivities'| 'valueForMoney'| 'embarkation'| 'dining'| 'publicRooms'| 'entertainment'| 'cabin'| 'fitnessAndRecreation'| 'shoreExcursions'| 'rates'| 'underThree'| 'threeToSix'| 'sevenToNine'| 'tenToTwelve'| 'thirteenToFifteen'| 'sixteenPlus'| 'service'| 'onboardExperience'| 'family';
  'ReviewsReviewCreateInput': {
    locale: string;
    countryCode: string | null;
    title: string;
    shipId: number;
    cruiseLineId: number;
    embarkationPortId: number;
    destinationId: number;
    cruiseLength: number;
    cruiseDay: number;
    cruiseMonth: number;
    cruiseYear: number;
    shipReview: string;
    overallRating: number;
    hasChildren: boolean;
    withDisabled: boolean | null;
    certification: boolean;
    numberOfCruisesTakenGroupId: number;
    providerId: number | null;
    entries: Array<Schema['ReviewsReviewEntryInput']>;
    cruiseStyles: Array<Schema['ReviewsReviewCruiseStyleInput']> | null;
  };
  'ReviewsReviewCruiseStyleInput': {
    cruiseStyleId: number;
  };
  'ReviewsReviewCruiseStyles': {
    __typename?: 'ReviewsReviewCruiseStyles';
    id: number;
    reviewId: number;
    cruiseStyleId: number;
    review?: Schema['ReviewsReviews'];
  };
  'ReviewsReviewEntries': {
    __typename?: 'ReviewsReviewEntries';
    id: number;
    reviewId: number | null;
    subjectId: number | null;
    subjectReferenceId: number | null;
    reviewCategory: Schema['ReviewsReviewCategory'] | null;
    status: number | null;
    rating: number | null;
    content: string | null;
    cabinPivot?: Array<Schema['ReviewsReviewCabinPivots']> | null;
    shorexPivot?: Array<Schema['ReviewsReviewShoreExcursionPivots']> | null;
    port?: Schema['DbPorts'] | null;
    shoreExcursion?: Schema['DbShoreExcursions'] | null;
  };
  'ReviewsReviewEntryInput': {
    reviewCategory: Schema['ReviewsReviewCategory'] | null;
    rating: number;
    content: string | null;
    portName: string | null;
    portId: number | null;
    cabinPivot: Schema['ReviewsReviewCabinPivotInput'] | null;
    shoreExcursionPivot: Schema['ReviewsReviewShoreExcursionPivotInput'] | null;
  };
  'ReviewsReviewFilterInput': {
    id: Array<number> | null;
    shipId: number | null;
    cruiseLineId: number | null;
    cruiseLineSlug: string | null;
    destinationId: number | null;
    destinationSlug: string | null;
    departurePortId: number | null;
    cruiseStyleId: number | null;
    cruiseStyleSlug: string | null;
    rating: number | null;
    reviewSlug: string | null;
    reviewSlugOwner: Schema['ReviewsReviewSlugOwners'] | null;
  };
  'ReviewsReviewShoreExcursionPivotInput': {
    isBookedWithCruiseLine: number | null;
    independentOperatorName: string | null;
    otherName: string | null;
    shoreExcursionId: number | null;
  };
  'ReviewsReviewShoreExcursionPivots': {
    __typename?: 'ReviewsReviewShoreExcursionPivots';
    id: number;
    reviewEntryId: number;
    isBookedWithCruiseLine: number | null;
    independentOperatorName: string | null;
    otherName: string | null;
    portId: number | null;
  };
  'ReviewsReviewSlugOwners': | 'cruiseLineDestination'| 'cruiseLineCruiseStyle'| 'cruiseLineDeparturePort'| 'cruiseLineCaribbean'| 'cruiseLineEurope'| 'cruiseLineMediterranean'| 'shipDestination'| 'destinationCruiseStyle'| 'shipCruiseStyle';
  'ReviewsReviewSlugs': {
    __typename?: 'ReviewsReviewSlugs';
    id: number;
    slug: string;
    reviewSlugOwner: Schema['ReviewsReviewSlugOwners'];
    cruiseLineId: number;
    destinationId: number;
    cruiseStyleId: number;
    shipId: number;
    departurePortId: number;
  };
  'ReviewsReviewUpdateInput': {
    title: string | null;
    cruisedOn: string | null;
  };
  'ReviewsReviews': {
    __typename?: 'ReviewsReviews';
    id: number;
    title: string | null;
    userName: string | null;
    userId: string | null;
    imsId: string | null;
    shipId: number | null;
    overallRating: number | null;
    embarkationPortId: number | null;
    destinationId: number | null;
    cruiseLength: number | null;
    cruiseDay: number | null;
    cruiseMonth: number | null;
    cruiseYear: number | null;
    cruisedOn: Schema['ReviewsDate'] | null;
    hasChildren: boolean | null;
    withDisabled: boolean;
    ip: string | null;
    countryCode: string | null;
    userAgent: string | null;
    certification: boolean | null;
    numberOfCruisesTakenGroupId: number | null;
    shipReview: string | null;
    shipReviewStatus: number | null;
    approvedBy: string | null;
    status: number | null;
    providerId: number | null;
    isPhotoJournal: boolean;
    variationId: string | null;
    publishedOn: Schema['ReviewsDate'] | null;
    publishedAt: Schema['ReviewsDateTime'] | null;
    createdAt: Schema['ReviewsDateTime'] | null;
    updatedAt: Schema['ReviewsDateTime'] | null;
    createdAtTs: number;
    updatedAtTs: number;
    publishedAtTs: number | null;
    ship?: Schema['ReviewsShips'];
    destination?: Schema['ReviewsDestinations'] | null;
    entries?: Array<Schema['ReviewsReviewEntries']> | null;
    cruiseStyles?: Array<Schema['ReviewsReviewCruiseStyles']> | null;
    reviewSummary: string | null;
    helpfulVotes: number;
    nextReview?: Schema['ReviewsReviews'] | null;
    previousReview?: Schema['ReviewsReviews'] | null;
    cabinCategoryCode: string | null;
    cabinCategory?: Schema['DbCabinCategoriesUnion'] | null;
    images?: Array<Schema['DbUserImages'] | null> | null;
    user?: Schema['DbSsoUser'] | null;
    departurePort?: Schema['DbDeparturePorts'] | null;
    itinerary?: Schema['DbItineraries'] | null;
    destinations?: Array<Schema['DbDestinations'] | null> | null;
  };
  'ReviewsShips': {
    __typename?: 'ReviewsShips';
    id: number;
    cruiseLineId: number;
    categoryRating?: number | null;
    maidenDate: string | null;
    maidenYear: number | null;
    primaryImage?: Schema['MetaImage'] | null;
    seo?: Schema['DbSeo'] | null;
    mappedImage?: Schema['DbImages'] | null;
    image: string | null;
    mappedImages?: Array<Schema['DbImageMappings'] | null> | null;
    snippets?: Array<Schema['DbShipSnippets'] | null> | null;
    hasUserPhotos: boolean | null;
    hasItineraries: boolean | null;
    snippetsForTypes?: Array<Schema['DbShipSnippets'] | null> | null;
    attributes?: Schema['DbShipAttributes'] | null;
    ratio: string | null;
    amenitiesByType?: Schema['DbShipAmenityResponse'] | null;
    destinations?: Array<Schema['DbDestinations'] | null> | null;
    ports?: Array<Schema['DbPorts'] | null> | null;
    pastSailings?: Array<Schema['DbStoredSailings'] | null> | null;
    cruisersChoiceAwards?: Array<Schema['DbCruisersChoiceCategories'] | null> | null;
    cruisersChoiceDestinationAwards?: Array<Schema['DbCruisersChoiceCategories'] | null> | null;
    editorsPicksAwards?: Array<Schema['DbEditorsPicksCategories'] | null> | null;
    editorsPicksResults?: Array<Schema['DbEditorsPicksResults'] | null> | null;
    cruiseStyles?: Array<Schema['DbCruiseStyles'] | null> | null;
    totalShoreExcursions: number | null;
  };
  'Reviews_Any': any;
  'Reviews_Entity': | Schema['ReviewsReviews'] | Schema['ReviewsShips'] | Schema['ReviewsDestinations'] | Schema['ReviewsReviewEntries'];
  'Reviews_Service': {
    __typename?: 'Reviews_Service';
  /**
   * The sdl representing the federated service capabilities. Includes federation directives, removes federation types, and includes rest of full schema after schema directives have been applied
   */
    sdl: string | null;
  };
  'SearchBestTimeToGo': {
    __typename?: 'SearchBestTimeToGo';
    isMostPopular: boolean;
    departureMonth: string;
    totalSailings: number;
    cabins?: Array<Schema['SearchBestTimeToGoCabins']>;
  };
  'SearchBestTimeToGoCabins': {
    __typename?: 'SearchBestTimeToGoCabins';
    cabinType?: Schema['SearchCabinTypes'];
    isLowestPrice: boolean;
    minPrice: number | null;
    maxPrice: number | null;
    avgPrice: number | null;
  };
  'SearchCabinType': | 'inside'| 'outside'| 'balcony'| 'suite';
  'SearchCabinTypes': {
    __typename?: 'SearchCabinTypes';
    id: number;
  };
  'SearchCruiseLength': {
    __typename?: 'SearchCruiseLength';
    id: string;
    name: string;
  };
  'SearchCruiseLineTier': | 'mainstream'| 'premium'| 'luxuryLite'| 'luxury';
  'SearchCruiseLines': {
    __typename?: 'SearchCruiseLines';
    id: number;
    name: string;
    shortName: string | null;
    iconUrl: string | null;
    logoUrl: string | null;
    tier: string | null;
    fragments: Array<string>;
    primaryImage?: Schema['SearchImage'] | null;
  };
  'SearchCruiseStyles': {
    __typename?: 'SearchCruiseStyles';
    findACruiseId: number | null;
  };
  'SearchCruisersChoice': {
    __typename?: 'SearchCruisersChoice';
    id: number;
    category: string;
    size: string;
    rating: number;
    totalReviews: number;
  };
  'SearchCurrency': | 'USD'| 'GBP'| 'AUD';
  /**
   * A date string, such as 2007-12-03, compliant with the `full-date` format outlined in section 5.6 of the RFC 3339 profile of the ISO 8601 standard for representation of dates and times using the Gregorian calendar.
   */
  'SearchDate': any;
  'SearchDealType': | 'all'| 'lastMinute';
  'SearchDepartureDate': {
    __typename?: 'SearchDepartureDate';
    id: string;
    name: string;
  };
  'SearchDepartureMonth': {
    __typename?: 'SearchDepartureMonth';
    id: string;
    name: string;
  };
  'SearchDeparturePorts': {
    __typename?: 'SearchDeparturePorts';
    id: number;
    name: string;
    portId: number | null;
  };
  'SearchDestinations': {
    __typename?: 'SearchDestinations';
    id: number;
    primaryImage?: Schema['SearchImage'] | null;
  };
  'SearchImage': {
    __typename?: 'SearchImage';
    id: number;
    format: string;
  };
  'SearchItineraries': {
    __typename?: 'SearchItineraries';
    id: number;
    title: string | null;
    length: number | null;
    score: number;
    sailings?: Array<Schema['SearchStoredSailings']>;
    cruiseLine?: Schema['SearchCruiseLines'];
    cruiseLineId: number;
    ship?: Schema['DbShips'] | null;
    shipId: number;
    destination?: Schema['DbDestinations'] | null;
    destinationId: number;
    itinerary?: Array<Schema['SearchItineraryPorts']>;
    portDeparture?: Schema['SearchPorts'];
    portArrival?: Schema['SearchPorts'];
    departurePorts?: Array<Schema['SearchDeparturePorts']>;
    leadInPrice?: Schema['SearchPrice'] | null;
  };
  'SearchItineraryDeal': {
    __typename?: 'SearchItineraryDeal';
    type: string;
    name: string;
  };
  'SearchItineraryPageFilterBonusOffer': {
    __typename?: 'SearchItineraryPageFilterBonusOffer';
    totalResults: number;
    value?: Schema['SearchProviderDealBenefitTypes'];
    lowestPrice: number | null;
  };
  'SearchItineraryPageFilterCabinType': {
    __typename?: 'SearchItineraryPageFilterCabinType';
    totalResults: number;
    value?: Schema['SearchCabinTypes'];
    lowestPrice: number | null;
  };
  'SearchItineraryPageFilterCruiseLine': {
    __typename?: 'SearchItineraryPageFilterCruiseLine';
    totalResults: number;
    value?: Schema['SearchCruiseLines'];
    lowestPrice: number | null;
  };
  'SearchItineraryPageFilterCruiseStyle': {
    __typename?: 'SearchItineraryPageFilterCruiseStyle';
    totalResults: number;
    value?: Schema['SearchCruiseStyles'];
    lowestPrice: number | null;
  };
  'SearchItineraryPageFilterDeal': {
    __typename?: 'SearchItineraryPageFilterDeal';
    totalResults: number;
    value?: Schema['SearchItineraryDeal'];
    lowestPrice: number | null;
  };
  'SearchItineraryPageFilterDepartureDate': {
    __typename?: 'SearchItineraryPageFilterDepartureDate';
    totalResults: number;
    value?: Schema['SearchDepartureDate'];
    lowestPrice: number | null;
  };
  'SearchItineraryPageFilterDepartureMonth': {
    __typename?: 'SearchItineraryPageFilterDepartureMonth';
    totalResults: number;
    value?: Schema['SearchDepartureMonth'];
    lowestPrice: number | null;
  };
  'SearchItineraryPageFilterDeparturePort': {
    __typename?: 'SearchItineraryPageFilterDeparturePort';
    totalResults: number;
    value?: Schema['SearchDeparturePorts'];
    lowestPrice: number | null;
  };
  'SearchItineraryPageFilterDestination': {
    __typename?: 'SearchItineraryPageFilterDestination';
    totalResults: number;
    value?: Schema['SearchDestinations'];
    lowestPrice: number | null;
  };
  'SearchItineraryPageFilterLength': {
    __typename?: 'SearchItineraryPageFilterLength';
    totalResults: number;
    value?: Schema['SearchCruiseLength'];
    lowestPrice: number | null;
  };
  'SearchItineraryPageFilterPackageType': {
    __typename?: 'SearchItineraryPageFilterPackageType';
    totalResults: number;
    value?: Schema['SearchPackageTypes'];
    lowestPrice: number | null;
  };
  'SearchItineraryPageFilterPort': {
    __typename?: 'SearchItineraryPageFilterPort';
    totalResults: number;
    value?: Schema['SearchPorts'];
    lowestPrice: number | null;
  };
  'SearchItineraryPageFilterPrice': {
    __typename?: 'SearchItineraryPageFilterPrice';
    min: number;
    max: number;
    lowestPrice: number;
  };
  'SearchItineraryPageFilterShip': {
    __typename?: 'SearchItineraryPageFilterShip';
    totalResults: number;
    value?: Schema['SearchShips'];
    lowestPrice: number | null;
  };
  'SearchItineraryPageFilters': {
    __typename?: 'SearchItineraryPageFilters';
    destinations?: Array<Schema['SearchItineraryPageFilterDestination']>;
    lengths?: Array<Schema['SearchItineraryPageFilterLength']>;
    cruiseStyles?: Array<Schema['SearchItineraryPageFilterCruiseStyle']>;
    cruiseLines?: Array<Schema['SearchItineraryPageFilterCruiseLine']>;
    ships?: Array<Schema['SearchItineraryPageFilterShip']>;
    departurePorts?: Array<Schema['SearchItineraryPageFilterDeparturePort']>;
    ports?: Array<Schema['SearchItineraryPageFilterPort']>;
    departureDates?: Array<Schema['SearchItineraryPageFilterDepartureDate']>;
    departureMonths?: Array<Schema['SearchItineraryPageFilterDepartureMonth']>;
    packageTypes?: Array<Schema['SearchItineraryPageFilterPackageType']>;
    cabinTypes?: Array<Schema['SearchItineraryPageFilterCabinType']>;
    bonusOffers?: Array<Schema['SearchItineraryPageFilterBonusOffer']>;
    deals?: Array<Schema['SearchItineraryPageFilterDeal']>;
    prices?: Schema['SearchItineraryPageFilterPrice'];
  };
  'SearchItineraryPorts': {
    __typename?: 'SearchItineraryPorts';
    day: number;
    port?: Schema['SearchPorts'];
  };
  'SearchItinerarySearchFiltersInput': {
    deals: Schema['SearchDealType'] | null;
    length: Array<string> | null;
    sailingId: Array<number> | null;
    packageType: Array<Schema['SearchPackageType']> | null;
    departureDate: string | null;
    hideSoldOut: boolean | null;
    vendorIds: Array<number> | null;
    destinationId: Array<number> | null;
    itineraryId: number | null;
    itineraryIds: Array<number> | null;
    cruiseLineId: Array<number> | null;
    shipId: Array<number> | null;
    portId: Array<number> | null;
    departurePortId: Array<number> | null;
    bonusOfferIds: Array<string> | null;
    cruiseStyleId: Array<number> | null;
    departureDateEnd: string | null;
    departureDateInterval: number | null;
    minPrice: number | null;
    maxPrice: number | null;
    cabinType: Schema['SearchCabinType'] | null;
    hasPricingForViewport: boolean | null;
    cruiseLineTier: Schema['SearchCruiseLineTier'] | null;
  };
  'SearchItinerarySearchResult': {
    __typename?: 'SearchItinerarySearchResult';
    currency: string;
    totalResults: number;
    results?: Array<Schema['SearchItineraries']>;
    statistics?: Schema['SearchItineraryStatistics'];
    pageFilters?: Schema['SearchItineraryPageFilters'];
  };
  'SearchItinerarySearchSortOrder': | 'popularity'| 'popularitySem'| 'score'| 'departureDate'| 'cruiseLine'| 'ship'| 'length'| 'rating'| 'price'| 'priceDesc';
  'SearchItineraryStatDeals': {
    __typename?: 'SearchItineraryStatDeals';
    maxDropPercentage: number;
  };
  'SearchItineraryStatPricingPerMonth': {
    __typename?: 'SearchItineraryStatPricingPerMonth';
    departureMonth: string;
    totalResults: number;
    minPrice: number | null;
    minPriceFormatted: string | null;
  };
  'SearchItineraryStatistics': {
    __typename?: 'SearchItineraryStatistics';
    deals?: Schema['SearchItineraryStatDeals'];
    pricingPerMonth?: Array<Schema['SearchItineraryStatPricingPerMonth']>;
    bestTimeToGo?: Array<Schema['SearchBestTimeToGo']>;
  };
  'SearchMutation': {
    __typename?: 'SearchMutation';
    indexArticle?: boolean;
    reindexArticles?: boolean;
  };
  'SearchPackageType': | 'cruiseOnly'| 'cruiseAndHotel'| 'cruiseAndFlight'| 'notApplicable';
  'SearchPackageTypes': {
    __typename?: 'SearchPackageTypes';
    type: Schema['SearchPackageType'];
  };
  'SearchPorts': {
    __typename?: 'SearchPorts';
    id: number;
    name: string;
    averageMemberRating: number | null;
    imageUrl: string | null;
    destinationId: number | null;
    longitude: string | null;
    latitude: string | null;
  };
  'SearchPosCountry': | 'AU'| 'GB'| 'US';
  'SearchPrice': {
    __typename?: 'SearchPrice';
    value: number;
    currency: Schema['SearchCurrency'];
  };
  'SearchProviderDealBenefitTypes': {
    __typename?: 'SearchProviderDealBenefitTypes';
    id: string;
  };
  'SearchProviderDealItinerariesResult': {
    __typename?: 'SearchProviderDealItinerariesResult';
    dealId: number;
    totalResults: number;
    results?: Array<Schema['SearchItineraries']>;
  };
  'SearchProviderDeals': {
    __typename?: 'SearchProviderDeals';
    id: number;
    countryId: number | null;
    sailingStartDate: Schema['SearchDate'] | null;
    sailingEndDate: Schema['SearchDate'] | null;
    sailingDateType: Schema['SearchProviderSailingDateType'];
    shipId: number;
    destinationId: number;
    length: number;
    itineraries?: Schema['SearchProviderDealItinerariesResult'] | null;
  };
  'SearchProviderSailingDateType': | 'simple'| 'range';
  'SearchQuery': {
    __typename?: 'SearchQuery';
    _entities?: Array<Schema['Search_Entity'] | null>;
    _service?: Schema['Search_Service'];
    indexing: string;
    itinerarySearch?: Schema['SearchItinerarySearchResult'];
    sailingPriceStats?: Schema['SearchSailingPriceStats'] | null;
    sailingPriceHistory?: Schema['SearchSailingPriceHistory'] | null;
    providerDealItineraries?: Schema['SearchProviderDealItinerariesResult'] | null;
  };
  'SearchSailingPriceHistory': {
    __typename?: 'SearchSailingPriceHistory';
    priceHistory?: Array<Schema['SearchSailingPriceHistoryResult']>;
  };
  'SearchSailingPriceHistoryResult': {
    __typename?: 'SearchSailingPriceHistoryResult';
    date: string;
    average: number;
    maximum: number;
    minimum: number;
    dealsCount: number;
  };
  'SearchSailingPriceStats': {
    __typename?: 'SearchSailingPriceStats';
    lowest: number;
    lowestDate: string;
    highest: number;
    highestDate: string;
    average: number;
  };
  'SearchSearchDeviceType': | 'MOBILE'| 'TABLET'| 'DESKTOP';
  'SearchShipAmenities': {
    __typename?: 'SearchShipAmenities';
    name: string;
    description: string | null;
    isIncluded: boolean;
  };
  'SearchShips': {
    __typename?: 'SearchShips';
    id: number;
    name: string;
    memberLovePercentage: number | null;
    averageMemberRating: number | null;
    professionalOverallRating: string | null;
    totalMemberReviews: number | null;
    imageUrl: string;
    fragments?: Array<string>;
    amenities?: Array<Schema['SearchShipAmenities']>;
    cruisersChoice?: Schema['SearchCruisersChoice'] | null;
    maidenDate: string | null;
    maidenYear: number | null;
    primaryImage?: Schema['MetaImage'] | null;
    seo?: Schema['DbSeo'] | null;
    mappedImage?: Schema['DbImages'] | null;
    image: string | null;
    mappedImages?: Array<Schema['DbImageMappings'] | null> | null;
    snippets?: Array<Schema['DbShipSnippets'] | null> | null;
    hasUserPhotos: boolean | null;
    hasItineraries: boolean | null;
    snippetsForTypes?: Array<Schema['DbShipSnippets'] | null> | null;
    attributes?: Schema['DbShipAttributes'] | null;
    ratio: string | null;
    amenitiesByType?: Schema['DbShipAmenityResponse'] | null;
    destinations?: Array<Schema['DbDestinations'] | null> | null;
    ports?: Array<Schema['DbPorts'] | null> | null;
    pastSailings?: Array<Schema['DbStoredSailings'] | null> | null;
    cruisersChoiceAwards?: Array<Schema['DbCruisersChoiceCategories'] | null> | null;
    cruisersChoiceDestinationAwards?: Array<Schema['DbCruisersChoiceCategories'] | null> | null;
    editorsPicksAwards?: Array<Schema['DbEditorsPicksCategories'] | null> | null;
    editorsPicksResults?: Array<Schema['DbEditorsPicksResults'] | null> | null;
    cruiseStyles?: Array<Schema['DbCruiseStyles'] | null> | null;
    totalShoreExcursions: number | null;
  };
  'SearchStoredSailings': {
    __typename?: 'SearchStoredSailings';
    id: number;
    departureDate: string;
  };
  'Search_Any': any;
  'Search_Entity': | Schema['SearchCabinTypes'] | Schema['SearchCruiseLines'] | Schema['SearchCruiseStyles'] | Schema['SearchDeparturePorts'] | Schema['SearchDestinations'] | Schema['SearchItineraries'] | Schema['SearchPackageTypes'] | Schema['SearchPorts'] | Schema['SearchProviderDealBenefitTypes'] | Schema['SearchProviderDeals'] | Schema['SearchShips'] | Schema['SearchStoredSailings'];
  'Search_Service': {
    __typename?: 'Search_Service';
  /**
   * The sdl representing the federated service capabilities. Includes federation directives, removes federation types, and includes rest of full schema after schema directives have been applied
   */
    sdl: string | null;
  };
  'SeoBaseNumbersInput': {
    crc_id: number;
    deck_number: number | null;
    review_count: number | null;
    result_count: number | null;
    photo_count: number | null;
    excursions_count: number | null;
  };
  'SeoBaseStringsInput': {
    year: string | null;
    price: string | null;
  };
  'SeoCruiseLineProfileInput': {
    year: string | null;
    price: string | null;
    cruise_line: string;
    seo_cruise_line: string;
    section: Schema['SeoCruiseLineProfileSection'] | null;
  };
  'SeoCruiseLineProfileSection': | 'OVERVIEW'| 'SHIPS'| 'ARTICLES'| 'NONE';
  'SeoDestProfileInput': {
    year: string | null;
    price: string | null;
    natural_destination_name: string | null;
    seo_destination_name: string | null;
    section: Schema['SeoDestProfileSection'] | null;
  };
  'SeoDestProfileSection': | 'OVERVIEW'| 'PORTS'| 'SHIPS'| 'ARTICLES'| 'NONE';
  'SeoFACInput': {
    year: string;
    price: string | null;
    cruise_line: string | null;
    seo_cruise_line: string | null;
    deal_subset: string | null;
    departure_date: string | null;
    destination_name: string | null;
    natural_destination_name: string | null;
    seo_destination_name: string | null;
    departure_port_name: string | null;
    seo_departure_port_name: string | null;
    state_abbreviation: string | null;
    itinerary_year: string | null;
    port_name: string | null;
    seo_port_name: string | null;
    port_state_abbreviation: string | null;
    cruise_length: string | null;
    ship_name: string | null;
    seo_ship_name: string | null;
    is_river: string | null;
    cruise_style_name: string | null;
    seo_cruise_style: string | null;
    lcase_cruise_style: string | null;
    lcase_destination_name: string | null;
    next_year: string | null;
    section: Schema['SeoFACSection'] | null;
  };
  'SeoFACSection': | 'RESULTS'| 'NONE';
  'SeoLocale': | 'en_UK'| 'en_US'| 'en_AU';
  'SeoOPF': {
    __typename?: 'SeoOPF';
    pageTitle: string | null;
    mastheadH1: string | null;
    mainH1: string | null;
    metaDescription: string | null;
    breadcrumb: string | null;
    altH1: string | null;
    logoAltText: string | null;
    metaPrepend: string | null;
    anchorText: string | null;
    name: string | null;
    factorName: string | null;
  };
  'SeoPortProfileInput': {
    year: string | null;
    price: string | null;
    seo_port_name: string | null;
    state_abbreviation: string | null;
    port_name: string | null;
    section: Schema['SeoPortProfileSection'] | null;
  };
  'SeoPortProfileSection': | 'OVERVIEW'| 'THINGS_TO_DO'| 'NONE';
  'SeoQuery': {
    __typename?: 'SeoQuery';
    _service?: Schema['Seo_Service'];
    cruiseLineProfile?: Schema['SeoOPF'] | null;
    destProfile?: Schema['SeoOPF'] | null;
    fac?: Schema['SeoOPF'] | null;
    portProfile?: Schema['SeoOPF'] | null;
    shipProfile?: Schema['SeoOPF'] | null;
    shorexProfile?: Schema['SeoOPF'] | null;
    home?: Schema['SeoOPF'] | null;
  };
  'SeoShipProfileInput': {
    year: string | null;
    price: string | null;
    deck_plan_name: string | null;
    ship_name: string | null;
    seo_ship_name: string | null;
    seo_cruise_line: string | null;
    cabin_type: string | null;
    cabin_type_slug: string | null;
    album_name: string | null;
    cabin_category_name: string | null;
    cabin_category_code: string | null;
    cabin_category_description: string | null;
    section: Schema['SeoShipProfileSection'] | null;
  };
  'SeoShipProfileSection': | 'OVERVIEW'| 'DINING'| 'ACTIVITIES'| 'PHOTOS'| 'CABINS'| 'CABIN_TYPE'| 'ALBUM'| 'SHIP_DECK_PLAN_LANDING'| 'SHIP_DECK_PLAN'| 'INDIVIDUAL_CABIN_CATEGORY'| 'LANDING'| 'NONE';
  'SeoShorexProfileInput': {
    year: string;
    price: string | null;
    port_name: string | null;
    seo_port_name: string | null;
    shorex_name: string | null;
    port_name_state: string | null;
    overview: string | null;
    section: Schema['SeoShorexProfileSection'] | null;
  };
  'SeoShorexProfileSection': | 'SHOREX_LANDING'| 'SHOREX_PORT'| 'SHOREX_INDIVIDUAL'| 'NONE';
  'Seo_Service': {
    __typename?: 'Seo_Service';
  /**
   * The sdl representing the federated service capabilities. Includes federation directives, removes federation types, and includes rest of full schema after schema directives have been applied
   */
    sdl: string | null;
  };
  'StoryblokAbtestComponent': {
    __typename?: 'StoryblokAbtestComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    enabled: boolean | null;
    jira_ticket_id: string | null;
    page_type: Array<string | null> | null;
    points_of_sale?: Array<Schema['StoryblokStory'] | null> | null;
    title: string | null;
    variants: Schema['StoryblokBlockScalar'] | null;
  };
  'StoryblokAbtestFilterQuery': {
    enabled: Schema['StoryblokFilterQueryOperations'] | null;
    title: Schema['StoryblokFilterQueryOperations'] | null;
    jira_ticket_id: Schema['StoryblokFilterQueryOperations'] | null;
    points_of_sale: Schema['StoryblokFilterQueryOperations'] | null;
    page_type: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokAbtestItem': {
    __typename?: 'StoryblokAbtestItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokAbtestComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokAbtestItems': {
    __typename?: 'StoryblokAbtestItems';
    items?: Array<Schema['StoryblokAbtestItem'] | null> | null;
    total: number | null;
  };
  'StoryblokAlternate': {
    __typename?: 'StoryblokAlternate';
    fullSlug: string;
    id: number;
    isFolder: boolean | null;
    name: string;
    parentId: number | null;
    published: boolean;
    slug: string;
  };
  'StoryblokArticleheroComponent': {
    __typename?: 'StoryblokArticleheroComponent';
    Hero_Image?: Schema['StoryblokAsset'] | null;
    Title: string | null;
    _editable: string | null;
    _uid: string | null;
    component: string | null;
  };
  'StoryblokArticleheroFilterQuery': {
    Title: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokArticleheroItem': {
    __typename?: 'StoryblokArticleheroItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokArticleheroComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokArticleheroItems': {
    __typename?: 'StoryblokArticleheroItems';
    items?: Array<Schema['StoryblokArticleheroItem'] | null> | null;
    total: number | null;
  };
  'StoryblokArticleweightedtagComponent': {
    __typename?: 'StoryblokArticleweightedtagComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    name: string | null;
    weight: string | null;
  };
  'StoryblokArticleweightedtagFilterQuery': {
    name: Schema['StoryblokFilterQueryOperations'] | null;
    weight: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokArticleweightedtagItem': {
    __typename?: 'StoryblokArticleweightedtagItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokArticleweightedtagComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokArticleweightedtagItems': {
    __typename?: 'StoryblokArticleweightedtagItems';
    items?: Array<Schema['StoryblokArticleweightedtagItem'] | null> | null;
    total: number | null;
  };
  'StoryblokAsset': {
    __typename?: 'StoryblokAsset';
    alt: string | null;
    copyright: string | null;
    filename: string;
    focus: string | null;
    id: number | null;
    name: string | null;
    title: string | null;
  };
  'StoryblokBlockScalar': any;
  'StoryblokChoiseawardcategoryComponent': {
    __typename?: 'StoryblokChoiseawardcategoryComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    image?: Schema['StoryblokAsset'] | null;
    name: string | null;
    title: string | null;
  };
  'StoryblokChoiseawardcategoryFilterQuery': {
    name: Schema['StoryblokFilterQueryOperations'] | null;
    title: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokChoiseawardcategoryItem': {
    __typename?: 'StoryblokChoiseawardcategoryItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokChoiseawardcategoryComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokChoiseawardcategoryItems': {
    __typename?: 'StoryblokChoiseawardcategoryItems';
    items?: Array<Schema['StoryblokChoiseawardcategoryItem'] | null> | null;
    total: number | null;
  };
  'StoryblokColorComponent': {
    __typename?: 'StoryblokColorComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    hex: string | null;
  };
  'StoryblokColorFilterQuery': {
    hex: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokColorItem': {
    __typename?: 'StoryblokColorItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokColorComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokColorItems': {
    __typename?: 'StoryblokColorItems';
    items?: Array<Schema['StoryblokColorItem'] | null> | null;
    total: number | null;
  };
  'StoryblokContentItem': {
    __typename?: 'StoryblokContentItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content: Schema['StoryblokJsonScalar'] | null;
    content_string: string | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokContentItems': {
    __typename?: 'StoryblokContentItems';
    items?: Array<Schema['StoryblokContentItem'] | null> | null;
    total: number | null;
  };
  'StoryblokCruisehealthandsafetyComponent': {
    __typename?: 'StoryblokCruisehealthandsafetyComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    external_id: string | null;
    title: string | null;
  };
  'StoryblokCruisehealthandsafetyFilterQuery': {
    title: Schema['StoryblokFilterQueryOperations'] | null;
    external_id: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokCruisehealthandsafetyItem': {
    __typename?: 'StoryblokCruisehealthandsafetyItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokCruisehealthandsafetyComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokCruisehealthandsafetyItems': {
    __typename?: 'StoryblokCruisehealthandsafetyItems';
    items?: Array<Schema['StoryblokCruisehealthandsafetyItem'] | null> | null;
    total: number | null;
  };
  'StoryblokCruiselineComponent': {
    __typename?: 'StoryblokCruiselineComponent';
    _editable: string | null;
    _uid: string | null;
    assets: Schema['StoryblokBlockScalar'] | null;
    component: string | null;
    external_id: string | null;
    gratuity_dollars: string | null;
    logo: Schema['StoryblokBlockScalar'] | null;
    name: string | null;
    round_logo: Schema['StoryblokBlockScalar'] | null;
    sales_name: string | null;
    seo_name: string | null;
  };
  'StoryblokCruiselineFilterQuery': {
    name: Schema['StoryblokFilterQueryOperations'] | null;
    seo_name: Schema['StoryblokFilterQueryOperations'] | null;
    sales_name: Schema['StoryblokFilterQueryOperations'] | null;
    external_id: Schema['StoryblokFilterQueryOperations'] | null;
    gratuity_dollars: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokCruiselineItem': {
    __typename?: 'StoryblokCruiselineItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokCruiselineComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokCruiselineItems': {
    __typename?: 'StoryblokCruiselineItems';
    items?: Array<Schema['StoryblokCruiselineItem'] | null> | null;
    total: number | null;
  };
  'StoryblokCruiserschoiceawardComponent': {
    __typename?: 'StoryblokCruiserschoiceawardComponent';
    _editable: string | null;
    _uid: string | null;
    category?: Schema['StoryblokStory'] | null;
    component: string | null;
    cruise_line_types: Schema['StoryblokBlockScalar'] | null;
    test: string | null;
  };
  'StoryblokCruiserschoiceawardFilterQuery': {
    category: Schema['StoryblokFilterQueryOperations'] | null;
    test: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokCruiserschoiceawardItem': {
    __typename?: 'StoryblokCruiserschoiceawardItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokCruiserschoiceawardComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokCruiserschoiceawardItems': {
    __typename?: 'StoryblokCruiserschoiceawardItems';
    items?: Array<Schema['StoryblokCruiserschoiceawardItem'] | null> | null;
    total: number | null;
  };
  'StoryblokCruiseshipComponent': {
    __typename?: 'StoryblokCruiseshipComponent';
    _editable: string | null;
    _uid: string | null;
    activities_rating: string | null;
    assets: Schema['StoryblokBlockScalar'] | null;
    cabins_rating: string | null;
    component: string | null;
    crew_total: string | null;
    cruise_line?: Schema['StoryblokStory'] | null;
    dining_rating: string | null;
    enrichment_rating: string | null;
    external_id: string | null;
    family_rating: string | null;
    fitness_recreation_rating: string | null;
    is_expedition: boolean | null;
    is_luxury: boolean | null;
    is_river: boolean | null;
    launch_year: string | null;
    name: string | null;
    overall_rating: string | null;
    passenger_to_crew_ratio: string | null;
    passengers_total: string | null;
    public_rooms_rating: string | null;
    sales_name: string | null;
    seo_name: string | null;
    service_rating: string | null;
    value_for_money_rating: string | null;
  };
  'StoryblokCruiseshipFilterQuery': {
    name: Schema['StoryblokFilterQueryOperations'] | null;
    cruise_line: Schema['StoryblokFilterQueryOperations'] | null;
    external_id: Schema['StoryblokFilterQueryOperations'] | null;
    passengers_total: Schema['StoryblokFilterQueryOperations'] | null;
    crew_total: Schema['StoryblokFilterQueryOperations'] | null;
    passenger_to_crew_ratio: Schema['StoryblokFilterQueryOperations'] | null;
    launch_year: Schema['StoryblokFilterQueryOperations'] | null;
    is_river: Schema['StoryblokFilterQueryOperations'] | null;
    is_luxury: Schema['StoryblokFilterQueryOperations'] | null;
    is_expedition: Schema['StoryblokFilterQueryOperations'] | null;
    seo_name: Schema['StoryblokFilterQueryOperations'] | null;
    sales_name: Schema['StoryblokFilterQueryOperations'] | null;
    overall_rating: Schema['StoryblokFilterQueryOperations'] | null;
    public_rooms_rating: Schema['StoryblokFilterQueryOperations'] | null;
    fitness_recreation_rating: Schema['StoryblokFilterQueryOperations'] | null;
    family_rating: Schema['StoryblokFilterQueryOperations'] | null;
    enrichment_rating: Schema['StoryblokFilterQueryOperations'] | null;
    service_rating: Schema['StoryblokFilterQueryOperations'] | null;
    value_for_money_rating: Schema['StoryblokFilterQueryOperations'] | null;
    cabins_rating: Schema['StoryblokFilterQueryOperations'] | null;
    dining_rating: Schema['StoryblokFilterQueryOperations'] | null;
    activities_rating: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokCruiseshipItem': {
    __typename?: 'StoryblokCruiseshipItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokCruiseshipComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokCruiseshipItems': {
    __typename?: 'StoryblokCruiseshipItems';
    items?: Array<Schema['StoryblokCruiseshipItem'] | null> | null;
    total: number | null;
  };
  'StoryblokCruisestylesComponent': {
    __typename?: 'StoryblokCruisestylesComponent';
    _editable: string | null;
    _uid: string | null;
    assets: Schema['StoryblokBlockScalar'] | null;
    component: string | null;
    description: Schema['StoryblokJsonScalar'] | null;
    external_id: string | null;
    fac_id: string | null;
    forum_id: string | null;
    name: string | null;
    snippets: Schema['StoryblokBlockScalar'] | null;
  };
  'StoryblokCruisestylesFilterQuery': {
    name: Schema['StoryblokFilterQueryOperations'] | null;
    external_id: Schema['StoryblokFilterQueryOperations'] | null;
    forum_id: Schema['StoryblokFilterQueryOperations'] | null;
    fac_id: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokCruisestylesItem': {
    __typename?: 'StoryblokCruisestylesItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokCruisestylesComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokCruisestylesItems': {
    __typename?: 'StoryblokCruisestylesItems';
    items?: Array<Schema['StoryblokCruisestylesItem'] | null> | null;
    total: number | null;
  };
  'StoryblokCustomheadlineComponent': {
    __typename?: 'StoryblokCustomheadlineComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    headline: string | null;
    headline_slug: string | null;
  };
  'StoryblokCustomheadlineFilterQuery': {
    headline: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokCustomheadlineItem': {
    __typename?: 'StoryblokCustomheadlineItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokCustomheadlineComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokCustomheadlineItems': {
    __typename?: 'StoryblokCustomheadlineItems';
    items?: Array<Schema['StoryblokCustomheadlineItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDatasource': {
    __typename?: 'StoryblokDatasource';
    id: number;
    name: string;
    slug: string;
  };
  'StoryblokDatasourceEntries': {
    __typename?: 'StoryblokDatasourceEntries';
    items?: Array<Schema['StoryblokDatasourceEntry']>;
    total: number;
  };
  'StoryblokDatasourceEntry': {
    __typename?: 'StoryblokDatasourceEntry';
    dimensionValue: string | null;
    id: number;
    name: string;
    value: string;
  };
  'StoryblokDatasources': {
    __typename?: 'StoryblokDatasources';
    items?: Array<Schema['StoryblokDatasource']>;
  };
  'StoryblokDepartureportoverviewComponent': {
    __typename?: 'StoryblokDepartureportoverviewComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    departure_port?: Schema['StoryblokStory'] | null;
    intro_body: Schema['StoryblokJsonScalar'] | null;
    intro_heading: string | null;
  };
  'StoryblokDepartureportoverviewFilterQuery': {
    departure_port: Schema['StoryblokFilterQueryOperations'] | null;
    intro_heading: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokDepartureportoverviewItem': {
    __typename?: 'StoryblokDepartureportoverviewItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokDepartureportoverviewComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDepartureportoverviewItems': {
    __typename?: 'StoryblokDepartureportoverviewItems';
    items?: Array<Schema['StoryblokDepartureportoverviewItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDepartureportsComponent': {
    __typename?: 'StoryblokDepartureportsComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    external_id: string | null;
    name: string | null;
    port?: Schema['StoryblokStory'] | null;
    seo_name: string | null;
    ta_location_id: string | null;
  };
  'StoryblokDepartureportsFilterQuery': {
    name: Schema['StoryblokFilterQueryOperations'] | null;
    seo_name: Schema['StoryblokFilterQueryOperations'] | null;
    external_id: Schema['StoryblokFilterQueryOperations'] | null;
    ta_location_id: Schema['StoryblokFilterQueryOperations'] | null;
    port: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokDepartureportsItem': {
    __typename?: 'StoryblokDepartureportsItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokDepartureportsComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDepartureportsItems': {
    __typename?: 'StoryblokDepartureportsItems';
    items?: Array<Schema['StoryblokDepartureportsItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDestinationoverviewComponent': {
    __typename?: 'StoryblokDestinationoverviewComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    destination?: Schema['StoryblokStory'] | null;
    intro_body: Schema['StoryblokJsonScalar'] | null;
    intro_heading: string | null;
    question_answer: Schema['StoryblokBlockScalar'] | null;
  };
  'StoryblokDestinationoverviewFilterQuery': {
    destination: Schema['StoryblokFilterQueryOperations'] | null;
    intro_heading: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokDestinationoverviewItem': {
    __typename?: 'StoryblokDestinationoverviewItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokDestinationoverviewComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDestinationoverviewItems': {
    __typename?: 'StoryblokDestinationoverviewItems';
    items?: Array<Schema['StoryblokDestinationoverviewItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDestinationsComponent': {
    __typename?: 'StoryblokDestinationsComponent';
    _editable: string | null;
    _uid: string | null;
    assets: Schema['StoryblokBlockScalar'] | null;
    component: string | null;
    external_id: string | null;
    hub_page?: Schema['StoryblokStory'] | null;
    is_river: boolean | null;
    name: string | null;
    sales_name: string | null;
    seo_name: string | null;
    ta_location_id: string | null;
  };
  'StoryblokDestinationsFilterQuery': {
    name: Schema['StoryblokFilterQueryOperations'] | null;
    sales_name: Schema['StoryblokFilterQueryOperations'] | null;
    hub_page: Schema['StoryblokFilterQueryOperations'] | null;
    external_id: Schema['StoryblokFilterQueryOperations'] | null;
    ta_location_id: Schema['StoryblokFilterQueryOperations'] | null;
    seo_name: Schema['StoryblokFilterQueryOperations'] | null;
    is_river: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokDestinationsItem': {
    __typename?: 'StoryblokDestinationsItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokDestinationsComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDestinationsItems': {
    __typename?: 'StoryblokDestinationsItems';
    items?: Array<Schema['StoryblokDestinationsItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDestinationslComponent': {
    __typename?: 'StoryblokDestinationslComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    contents: Schema['StoryblokBlockScalar'] | null;
    seo: Schema['StoryblokJsonScalar'] | null;
  };
  'StoryblokDestinationslItem': {
    __typename?: 'StoryblokDestinationslItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokDestinationslComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDestinationslItems': {
    __typename?: 'StoryblokDestinationslItems';
    items?: Array<Schema['StoryblokDestinationslItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftAbtestComponent': {
    __typename?: 'StoryblokDraftAbtestComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    enabled: boolean | null;
    jira_ticket_id: string | null;
    page_type: Array<string | null> | null;
    points_of_sale?: Array<Schema['StoryblokDraftStory'] | null> | null;
    title: string | null;
    variants: Schema['StoryblokDraftBlockScalar'] | null;
  };
  'StoryblokDraftAbtestFilterQuery': {
    enabled: Schema['StoryblokDraftFilterQueryOperations'] | null;
    title: Schema['StoryblokDraftFilterQueryOperations'] | null;
    jira_ticket_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    points_of_sale: Schema['StoryblokDraftFilterQueryOperations'] | null;
    page_type: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftAbtestItem': {
    __typename?: 'StoryblokDraftAbtestItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftAbtestComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftAbtestItems': {
    __typename?: 'StoryblokDraftAbtestItems';
    items?: Array<Schema['StoryblokDraftAbtestItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftAlternate': {
    __typename?: 'StoryblokDraftAlternate';
    fullSlug: string;
    id: number;
    isFolder: boolean | null;
    name: string;
    parentId: number | null;
    published: boolean;
    slug: string;
  };
  'StoryblokDraftArticleheroComponent': {
    __typename?: 'StoryblokDraftArticleheroComponent';
    Hero_Image?: Schema['StoryblokDraftAsset'] | null;
    Title: string | null;
    _editable: string | null;
    _uid: string | null;
    component: string | null;
  };
  'StoryblokDraftArticleheroFilterQuery': {
    Title: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftArticleheroItem': {
    __typename?: 'StoryblokDraftArticleheroItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftArticleheroComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftArticleheroItems': {
    __typename?: 'StoryblokDraftArticleheroItems';
    items?: Array<Schema['StoryblokDraftArticleheroItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftArticleweightedtagComponent': {
    __typename?: 'StoryblokDraftArticleweightedtagComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    name: string | null;
    weight: string | null;
  };
  'StoryblokDraftArticleweightedtagFilterQuery': {
    name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    weight: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftArticleweightedtagItem': {
    __typename?: 'StoryblokDraftArticleweightedtagItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftArticleweightedtagComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftArticleweightedtagItems': {
    __typename?: 'StoryblokDraftArticleweightedtagItems';
    items?: Array<Schema['StoryblokDraftArticleweightedtagItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftAsset': {
    __typename?: 'StoryblokDraftAsset';
    alt: string | null;
    copyright: string | null;
    filename: string;
    focus: string | null;
    id: number | null;
    name: string | null;
    title: string | null;
  };
  'StoryblokDraftBlockScalar': any;
  'StoryblokDraftChoiseawardcategoryComponent': {
    __typename?: 'StoryblokDraftChoiseawardcategoryComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    image?: Schema['StoryblokDraftAsset'] | null;
    name: string | null;
    title: string | null;
  };
  'StoryblokDraftChoiseawardcategoryFilterQuery': {
    name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    title: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftChoiseawardcategoryItem': {
    __typename?: 'StoryblokDraftChoiseawardcategoryItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftChoiseawardcategoryComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftChoiseawardcategoryItems': {
    __typename?: 'StoryblokDraftChoiseawardcategoryItems';
    items?: Array<Schema['StoryblokDraftChoiseawardcategoryItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftColorComponent': {
    __typename?: 'StoryblokDraftColorComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    hex: string | null;
  };
  'StoryblokDraftColorFilterQuery': {
    hex: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftColorItem': {
    __typename?: 'StoryblokDraftColorItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftColorComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftColorItems': {
    __typename?: 'StoryblokDraftColorItems';
    items?: Array<Schema['StoryblokDraftColorItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftContentItem': {
    __typename?: 'StoryblokDraftContentItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content: Schema['StoryblokDraftJsonScalar'] | null;
    content_string: string | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftContentItems': {
    __typename?: 'StoryblokDraftContentItems';
    items?: Array<Schema['StoryblokDraftContentItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftCruisehealthandsafetyComponent': {
    __typename?: 'StoryblokDraftCruisehealthandsafetyComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    external_id: string | null;
    title: string | null;
  };
  'StoryblokDraftCruisehealthandsafetyFilterQuery': {
    title: Schema['StoryblokDraftFilterQueryOperations'] | null;
    external_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftCruisehealthandsafetyItem': {
    __typename?: 'StoryblokDraftCruisehealthandsafetyItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftCruisehealthandsafetyComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftCruisehealthandsafetyItems': {
    __typename?: 'StoryblokDraftCruisehealthandsafetyItems';
    items?: Array<Schema['StoryblokDraftCruisehealthandsafetyItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftCruiselineComponent': {
    __typename?: 'StoryblokDraftCruiselineComponent';
    _editable: string | null;
    _uid: string | null;
    assets: Schema['StoryblokDraftBlockScalar'] | null;
    component: string | null;
    external_id: string | null;
    gratuity_dollars: string | null;
    logo: Schema['StoryblokDraftBlockScalar'] | null;
    name: string | null;
    round_logo: Schema['StoryblokDraftBlockScalar'] | null;
    sales_name: string | null;
    seo_name: string | null;
  };
  'StoryblokDraftCruiselineFilterQuery': {
    name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    seo_name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    sales_name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    external_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    gratuity_dollars: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftCruiselineItem': {
    __typename?: 'StoryblokDraftCruiselineItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftCruiselineComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftCruiselineItems': {
    __typename?: 'StoryblokDraftCruiselineItems';
    items?: Array<Schema['StoryblokDraftCruiselineItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftCruiserschoiceawardComponent': {
    __typename?: 'StoryblokDraftCruiserschoiceawardComponent';
    _editable: string | null;
    _uid: string | null;
    category?: Schema['StoryblokDraftStory'] | null;
    component: string | null;
    cruise_line_types: Schema['StoryblokDraftBlockScalar'] | null;
    test: string | null;
  };
  'StoryblokDraftCruiserschoiceawardFilterQuery': {
    category: Schema['StoryblokDraftFilterQueryOperations'] | null;
    test: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftCruiserschoiceawardItem': {
    __typename?: 'StoryblokDraftCruiserschoiceawardItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftCruiserschoiceawardComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftCruiserschoiceawardItems': {
    __typename?: 'StoryblokDraftCruiserschoiceawardItems';
    items?: Array<Schema['StoryblokDraftCruiserschoiceawardItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftCruiseshipComponent': {
    __typename?: 'StoryblokDraftCruiseshipComponent';
    _editable: string | null;
    _uid: string | null;
    activities_rating: string | null;
    assets: Schema['StoryblokDraftBlockScalar'] | null;
    cabins_rating: string | null;
    component: string | null;
    crew_total: string | null;
    cruise_line?: Schema['StoryblokDraftStory'] | null;
    dining_rating: string | null;
    enrichment_rating: string | null;
    external_id: string | null;
    family_rating: string | null;
    fitness_recreation_rating: string | null;
    is_expedition: boolean | null;
    is_luxury: boolean | null;
    is_river: boolean | null;
    launch_year: string | null;
    name: string | null;
    overall_rating: string | null;
    passenger_to_crew_ratio: string | null;
    passengers_total: string | null;
    public_rooms_rating: string | null;
    sales_name: string | null;
    seo_name: string | null;
    service_rating: string | null;
    value_for_money_rating: string | null;
  };
  'StoryblokDraftCruiseshipFilterQuery': {
    name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    cruise_line: Schema['StoryblokDraftFilterQueryOperations'] | null;
    external_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    passengers_total: Schema['StoryblokDraftFilterQueryOperations'] | null;
    crew_total: Schema['StoryblokDraftFilterQueryOperations'] | null;
    passenger_to_crew_ratio: Schema['StoryblokDraftFilterQueryOperations'] | null;
    launch_year: Schema['StoryblokDraftFilterQueryOperations'] | null;
    is_river: Schema['StoryblokDraftFilterQueryOperations'] | null;
    is_luxury: Schema['StoryblokDraftFilterQueryOperations'] | null;
    is_expedition: Schema['StoryblokDraftFilterQueryOperations'] | null;
    seo_name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    sales_name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    overall_rating: Schema['StoryblokDraftFilterQueryOperations'] | null;
    public_rooms_rating: Schema['StoryblokDraftFilterQueryOperations'] | null;
    fitness_recreation_rating: Schema['StoryblokDraftFilterQueryOperations'] | null;
    family_rating: Schema['StoryblokDraftFilterQueryOperations'] | null;
    enrichment_rating: Schema['StoryblokDraftFilterQueryOperations'] | null;
    service_rating: Schema['StoryblokDraftFilterQueryOperations'] | null;
    value_for_money_rating: Schema['StoryblokDraftFilterQueryOperations'] | null;
    cabins_rating: Schema['StoryblokDraftFilterQueryOperations'] | null;
    dining_rating: Schema['StoryblokDraftFilterQueryOperations'] | null;
    activities_rating: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftCruiseshipItem': {
    __typename?: 'StoryblokDraftCruiseshipItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftCruiseshipComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftCruiseshipItems': {
    __typename?: 'StoryblokDraftCruiseshipItems';
    items?: Array<Schema['StoryblokDraftCruiseshipItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftCruisestylesComponent': {
    __typename?: 'StoryblokDraftCruisestylesComponent';
    _editable: string | null;
    _uid: string | null;
    assets: Schema['StoryblokDraftBlockScalar'] | null;
    component: string | null;
    description: Schema['StoryblokDraftJsonScalar'] | null;
    external_id: string | null;
    fac_id: string | null;
    forum_id: string | null;
    name: string | null;
    snippets: Schema['StoryblokDraftBlockScalar'] | null;
  };
  'StoryblokDraftCruisestylesFilterQuery': {
    name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    external_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    forum_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    fac_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftCruisestylesItem': {
    __typename?: 'StoryblokDraftCruisestylesItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftCruisestylesComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftCruisestylesItems': {
    __typename?: 'StoryblokDraftCruisestylesItems';
    items?: Array<Schema['StoryblokDraftCruisestylesItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftCustomheadlineComponent': {
    __typename?: 'StoryblokDraftCustomheadlineComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    headline: string | null;
    headline_slug: string | null;
  };
  'StoryblokDraftCustomheadlineFilterQuery': {
    headline: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftCustomheadlineItem': {
    __typename?: 'StoryblokDraftCustomheadlineItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftCustomheadlineComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftCustomheadlineItems': {
    __typename?: 'StoryblokDraftCustomheadlineItems';
    items?: Array<Schema['StoryblokDraftCustomheadlineItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftDatasource': {
    __typename?: 'StoryblokDraftDatasource';
    id: number;
    name: string;
    slug: string;
  };
  'StoryblokDraftDatasourceEntries': {
    __typename?: 'StoryblokDraftDatasourceEntries';
    items?: Array<Schema['StoryblokDraftDatasourceEntry']>;
    total: number;
  };
  'StoryblokDraftDatasourceEntry': {
    __typename?: 'StoryblokDraftDatasourceEntry';
    dimensionValue: string | null;
    id: number;
    name: string;
    value: string;
  };
  'StoryblokDraftDatasources': {
    __typename?: 'StoryblokDraftDatasources';
    items?: Array<Schema['StoryblokDraftDatasource']>;
  };
  'StoryblokDraftDepartureportoverviewComponent': {
    __typename?: 'StoryblokDraftDepartureportoverviewComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    departure_port?: Schema['StoryblokDraftStory'] | null;
    intro_body: Schema['StoryblokDraftJsonScalar'] | null;
    intro_heading: string | null;
  };
  'StoryblokDraftDepartureportoverviewFilterQuery': {
    departure_port: Schema['StoryblokDraftFilterQueryOperations'] | null;
    intro_heading: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftDepartureportoverviewItem': {
    __typename?: 'StoryblokDraftDepartureportoverviewItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftDepartureportoverviewComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftDepartureportoverviewItems': {
    __typename?: 'StoryblokDraftDepartureportoverviewItems';
    items?: Array<Schema['StoryblokDraftDepartureportoverviewItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftDepartureportsComponent': {
    __typename?: 'StoryblokDraftDepartureportsComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    external_id: string | null;
    name: string | null;
    port?: Schema['StoryblokDraftStory'] | null;
    seo_name: string | null;
    ta_location_id: string | null;
  };
  'StoryblokDraftDepartureportsFilterQuery': {
    name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    seo_name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    external_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    ta_location_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    port: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftDepartureportsItem': {
    __typename?: 'StoryblokDraftDepartureportsItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftDepartureportsComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftDepartureportsItems': {
    __typename?: 'StoryblokDraftDepartureportsItems';
    items?: Array<Schema['StoryblokDraftDepartureportsItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftDestinationoverviewComponent': {
    __typename?: 'StoryblokDraftDestinationoverviewComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    destination?: Schema['StoryblokDraftStory'] | null;
    intro_body: Schema['StoryblokDraftJsonScalar'] | null;
    intro_heading: string | null;
    question_answer: Schema['StoryblokDraftBlockScalar'] | null;
  };
  'StoryblokDraftDestinationoverviewFilterQuery': {
    destination: Schema['StoryblokDraftFilterQueryOperations'] | null;
    intro_heading: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftDestinationoverviewItem': {
    __typename?: 'StoryblokDraftDestinationoverviewItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftDestinationoverviewComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftDestinationoverviewItems': {
    __typename?: 'StoryblokDraftDestinationoverviewItems';
    items?: Array<Schema['StoryblokDraftDestinationoverviewItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftDestinationsComponent': {
    __typename?: 'StoryblokDraftDestinationsComponent';
    _editable: string | null;
    _uid: string | null;
    assets: Schema['StoryblokDraftBlockScalar'] | null;
    component: string | null;
    external_id: string | null;
    hub_page?: Schema['StoryblokDraftStory'] | null;
    is_river: boolean | null;
    name: string | null;
    sales_name: string | null;
    seo_name: string | null;
    ta_location_id: string | null;
  };
  'StoryblokDraftDestinationsFilterQuery': {
    name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    sales_name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    hub_page: Schema['StoryblokDraftFilterQueryOperations'] | null;
    external_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    ta_location_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    seo_name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    is_river: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftDestinationsItem': {
    __typename?: 'StoryblokDraftDestinationsItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftDestinationsComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftDestinationsItems': {
    __typename?: 'StoryblokDraftDestinationsItems';
    items?: Array<Schema['StoryblokDraftDestinationsItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftDestinationslComponent': {
    __typename?: 'StoryblokDraftDestinationslComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    contents: Schema['StoryblokDraftBlockScalar'] | null;
    seo: Schema['StoryblokDraftJsonScalar'] | null;
  };
  'StoryblokDraftDestinationslItem': {
    __typename?: 'StoryblokDraftDestinationslItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftDestinationslComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftDestinationslItems': {
    __typename?: 'StoryblokDraftDestinationslItems';
    items?: Array<Schema['StoryblokDraftDestinationslItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorialarticlehubComponent': {
    __typename?: 'StoryblokDraftEditorialarticlehubComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
  };
  'StoryblokDraftEditorialarticlehubItem': {
    __typename?: 'StoryblokDraftEditorialarticlehubItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorialarticlehubComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorialarticlehubItems': {
    __typename?: 'StoryblokDraftEditorialarticlehubItems';
    items?: Array<Schema['StoryblokDraftEditorialarticlehubItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorialauthorsComponent': {
    __typename?: 'StoryblokDraftEditorialauthorsComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    image?: Schema['StoryblokDraftAsset'] | null;
    name: string | null;
    title: string | null;
  };
  'StoryblokDraftEditorialauthorsFilterQuery': {
    name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    title: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftEditorialauthorsItem': {
    __typename?: 'StoryblokDraftEditorialauthorsItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorialauthorsComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorialauthorsItems': {
    __typename?: 'StoryblokDraftEditorialauthorsItems';
    items?: Array<Schema['StoryblokDraftEditorialauthorsItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorialcontentComponent': {
    __typename?: 'StoryblokDraftEditorialcontentComponent';
    _editable: string | null;
    _uid: string | null;
    body: Schema['StoryblokDraftJsonScalar'] | null;
    component: string | null;
  };
  'StoryblokDraftEditorialcontentItem': {
    __typename?: 'StoryblokDraftEditorialcontentItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorialcontentComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorialcontentItems': {
    __typename?: 'StoryblokDraftEditorialcontentItems';
    items?: Array<Schema['StoryblokDraftEditorialcontentItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorialcruiselineoverviewComponent': {
    __typename?: 'StoryblokDraftEditorialcruiselineoverviewComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    intro: Schema['StoryblokDraftJsonScalar'] | null;
    partner_message: Schema['StoryblokDraftBlockScalar'] | null;
    questions_answers: Schema['StoryblokDraftBlockScalar'] | null;
  };
  'StoryblokDraftEditorialcruiselineoverviewItem': {
    __typename?: 'StoryblokDraftEditorialcruiselineoverviewItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorialcruiselineoverviewComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorialcruiselineoverviewItems': {
    __typename?: 'StoryblokDraftEditorialcruiselineoverviewItems';
    items?: Array<Schema['StoryblokDraftEditorialcruiselineoverviewItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorialcruiseshipactivitiesComponent': {
    __typename?: 'StoryblokDraftEditorialcruiseshipactivitiesComponent';
    _editable: string | null;
    _uid: string | null;
    activities_and_entertainment: Schema['StoryblokDraftBlockScalar'] | null;
    author?: Schema['StoryblokDraftStory'] | null;
    body: Schema['StoryblokDraftBlockScalar'] | null;
    component: string | null;
    editorial_rating: Schema['StoryblokDraftBlockScalar'] | null;
    header_assets: Schema['StoryblokDraftBlockScalar'] | null;
    metatags: Schema['StoryblokDraftJsonScalar'] | null;
    ship?: Schema['StoryblokDraftStory'] | null;
  };
  'StoryblokDraftEditorialcruiseshipactivitiesFilterQuery': {
    ship: Schema['StoryblokDraftFilterQueryOperations'] | null;
    author: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftEditorialcruiseshipactivitiesItem': {
    __typename?: 'StoryblokDraftEditorialcruiseshipactivitiesItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorialcruiseshipactivitiesComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorialcruiseshipactivitiesItems': {
    __typename?: 'StoryblokDraftEditorialcruiseshipactivitiesItems';
    items?: Array<Schema['StoryblokDraftEditorialcruiseshipactivitiesItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorialcruiseshipcabinComponent': {
    __typename?: 'StoryblokDraftEditorialcruiseshipcabinComponent';
    _editable: string | null;
    _uid: string | null;
    author?: Schema['StoryblokDraftStory'] | null;
    category_assets: Schema['StoryblokDraftBlockScalar'] | null;
    component: string | null;
    editorial_rating: Schema['StoryblokDraftBlockScalar'] | null;
    header_assets: Schema['StoryblokDraftBlockScalar'] | null;
    intro: Schema['StoryblokDraftBlockScalar'] | null;
    metatags: Schema['StoryblokDraftJsonScalar'] | null;
    ship?: Schema['StoryblokDraftStory'] | null;
    text: Schema['StoryblokDraftBlockScalar'] | null;
  };
  'StoryblokDraftEditorialcruiseshipcabinFilterQuery': {
    ship: Schema['StoryblokDraftFilterQueryOperations'] | null;
    author: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftEditorialcruiseshipcabinItem': {
    __typename?: 'StoryblokDraftEditorialcruiseshipcabinItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorialcruiseshipcabinComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorialcruiseshipcabinItems': {
    __typename?: 'StoryblokDraftEditorialcruiseshipcabinItems';
    items?: Array<Schema['StoryblokDraftEditorialcruiseshipcabinItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorialcruiseshipdeckplanComponent': {
    __typename?: 'StoryblokDraftEditorialcruiseshipdeckplanComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    decks: Schema['StoryblokDraftBlockScalar'] | null;
    metatags: Schema['StoryblokDraftJsonScalar'] | null;
    ship?: Schema['StoryblokDraftStory'] | null;
  };
  'StoryblokDraftEditorialcruiseshipdeckplanFilterQuery': {
    ship: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftEditorialcruiseshipdeckplanItem': {
    __typename?: 'StoryblokDraftEditorialcruiseshipdeckplanItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorialcruiseshipdeckplanComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorialcruiseshipdeckplanItems': {
    __typename?: 'StoryblokDraftEditorialcruiseshipdeckplanItems';
    items?: Array<Schema['StoryblokDraftEditorialcruiseshipdeckplanItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorialcruiseshipdiningComponent': {
    __typename?: 'StoryblokDraftEditorialcruiseshipdiningComponent';
    _editable: string | null;
    _uid: string | null;
    author?: Schema['StoryblokDraftStory'] | null;
    body: Schema['StoryblokDraftBlockScalar'] | null;
    component: string | null;
    editorial_rating: Schema['StoryblokDraftBlockScalar'] | null;
    header_assets: Schema['StoryblokDraftBlockScalar'] | null;
    metatags: Schema['StoryblokDraftJsonScalar'] | null;
    restaurants: Schema['StoryblokDraftBlockScalar'] | null;
    ship?: Schema['StoryblokDraftStory'] | null;
  };
  'StoryblokDraftEditorialcruiseshipdiningFilterQuery': {
    ship: Schema['StoryblokDraftFilterQueryOperations'] | null;
    author: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftEditorialcruiseshipdiningItem': {
    __typename?: 'StoryblokDraftEditorialcruiseshipdiningItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorialcruiseshipdiningComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorialcruiseshipdiningItems': {
    __typename?: 'StoryblokDraftEditorialcruiseshipdiningItems';
    items?: Array<Schema['StoryblokDraftEditorialcruiseshipdiningItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorialcruiseshipoverviewComponent': {
    __typename?: 'StoryblokDraftEditorialcruiseshipoverviewComponent';
    _editable: string | null;
    _uid: string | null;
    author?: Schema['StoryblokDraftStory'] | null;
    component: string | null;
    dress_codes: Schema['StoryblokDraftBlockScalar'] | null;
    editorial_rating: Schema['StoryblokDraftBlockScalar'] | null;
    exclusions: Schema['StoryblokDraftBlockScalar'] | null;
    exclusions_text: string | null;
    fellow_passengers: Schema['StoryblokDraftBlockScalar'] | null;
    header_assets: Schema['StoryblokDraftBlockScalar'] | null;
    inclusions: Schema['StoryblokDraftBlockScalar'] | null;
    inclusions_text: string | null;
    intro: Schema['StoryblokDraftBlockScalar'] | null;
    metatags: Schema['StoryblokDraftJsonScalar'] | null;
    metatags_variation1: Schema['StoryblokDraftJsonScalar'] | null;
    metatags_variation2: Schema['StoryblokDraftJsonScalar'] | null;
    overview: Schema['StoryblokDraftBlockScalar'] | null;
    review_highlights: Schema['StoryblokDraftBlockScalar'] | null;
    ship?: Schema['StoryblokDraftStory'] | null;
  };
  'StoryblokDraftEditorialcruiseshipoverviewFilterQuery': {
    ship: Schema['StoryblokDraftFilterQueryOperations'] | null;
    author: Schema['StoryblokDraftFilterQueryOperations'] | null;
    inclusions_text: Schema['StoryblokDraftFilterQueryOperations'] | null;
    exclusions_text: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftEditorialcruiseshipoverviewItem': {
    __typename?: 'StoryblokDraftEditorialcruiseshipoverviewItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorialcruiseshipoverviewComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorialcruiseshipoverviewItems': {
    __typename?: 'StoryblokDraftEditorialcruiseshipoverviewItems';
    items?: Array<Schema['StoryblokDraftEditorialcruiseshipoverviewItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorialfeaturearticleComponent': {
    __typename?: 'StoryblokDraftEditorialfeaturearticleComponent';
    _editable: string | null;
    _uid: string | null;
    area_id?: Schema['StoryblokDraftStory'] | null;
    author?: Array<Schema['StoryblokDraftStory'] | null> | null;
    body: Schema['StoryblokDraftBlockScalar'] | null;
    client_key: string | null;
    component: string | null;
    disable_on_pos: Array<string | null> | null;
    external_id: string | null;
    gpt_ad_overide?: Schema['StoryblokDraftStory'] | null;
    headline: Schema['StoryblokDraftBlockScalar'] | null;
    hero_image: Schema['StoryblokDraftBlockScalar'] | null;
    is_featured_content_enabled: boolean | null;
    is_negative: boolean | null;
    is_no_index: boolean | null;
    keywords: string | null;
    metatags: Schema['StoryblokDraftJsonScalar'] | null;
    primary_area: string | null;
    promo: Schema['StoryblokDraftBlockScalar'] | null;
    sponsored_content_target: string | null;
    syndication_id: string | null;
    table_of_contents_type: string | null;
    tags?: Array<Schema['StoryblokDraftStory'] | null> | null;
    updated_date: string | null;
  };
  'StoryblokDraftEditorialfeaturearticleFilterQuery': {
    author: Schema['StoryblokDraftFilterQueryOperations'] | null;
    gpt_ad_overide: Schema['StoryblokDraftFilterQueryOperations'] | null;
    table_of_contents_type: Schema['StoryblokDraftFilterQueryOperations'] | null;
    primary_area: Schema['StoryblokDraftFilterQueryOperations'] | null;
    sponsored_content_target: Schema['StoryblokDraftFilterQueryOperations'] | null;
    is_featured_content_enabled: Schema['StoryblokDraftFilterQueryOperations'] | null;
    is_negative: Schema['StoryblokDraftFilterQueryOperations'] | null;
    disable_on_pos: Schema['StoryblokDraftFilterQueryOperations'] | null;
    tags: Schema['StoryblokDraftFilterQueryOperations'] | null;
    area_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    syndication_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    external_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    client_key: Schema['StoryblokDraftFilterQueryOperations'] | null;
    is_no_index: Schema['StoryblokDraftFilterQueryOperations'] | null;
    keywords: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftEditorialfeaturearticleItem': {
    __typename?: 'StoryblokDraftEditorialfeaturearticleItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorialfeaturearticleComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorialfeaturearticleItems': {
    __typename?: 'StoryblokDraftEditorialfeaturearticleItems';
    items?: Array<Schema['StoryblokDraftEditorialfeaturearticleItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorialfeaturearticleheroComponent': {
    __typename?: 'StoryblokDraftEditorialfeaturearticleheroComponent';
    _editable: string | null;
    _uid: string | null;
    article?: Schema['StoryblokDraftStory'] | null;
    component: string | null;
  };
  'StoryblokDraftEditorialfeaturearticleheroFilterQuery': {
    article: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftEditorialfeaturearticleheroItem': {
    __typename?: 'StoryblokDraftEditorialfeaturearticleheroItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorialfeaturearticleheroComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorialfeaturearticleheroItems': {
    __typename?: 'StoryblokDraftEditorialfeaturearticleheroItems';
    items?: Array<Schema['StoryblokDraftEditorialfeaturearticleheroItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorialfeaturearticlelandingComponent': {
    __typename?: 'StoryblokDraftEditorialfeaturearticlelandingComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    editors_picks_articles: Schema['StoryblokDraftBlockScalar'] | null;
    hero_article: Schema['StoryblokDraftBlockScalar'] | null;
  };
  'StoryblokDraftEditorialfeaturearticlelandingItem': {
    __typename?: 'StoryblokDraftEditorialfeaturearticlelandingItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorialfeaturearticlelandingComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorialfeaturearticlelandingItems': {
    __typename?: 'StoryblokDraftEditorialfeaturearticlelandingItems';
    items?: Array<Schema['StoryblokDraftEditorialfeaturearticlelandingItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorialnewsarticleComponent': {
    __typename?: 'StoryblokDraftEditorialnewsarticleComponent';
    _editable: string | null;
    _uid: string | null;
    area_id?: Schema['StoryblokDraftStory'] | null;
    author?: Array<Schema['StoryblokDraftStory'] | null> | null;
    body: Schema['StoryblokDraftBlockScalar'] | null;
    client_key: string | null;
    component: string | null;
    disable_on_pos: Array<string | null> | null;
    external_id: string | null;
    gpt_ad_overide?: Schema['StoryblokDraftStory'] | null;
    headline: Schema['StoryblokDraftBlockScalar'] | null;
    hero_image: Schema['StoryblokDraftBlockScalar'] | null;
    is_featured_content_enabled: boolean | null;
    is_negative: boolean | null;
    is_no_index: boolean | null;
    metatags: Schema['StoryblokDraftJsonScalar'] | null;
    primary_area: string | null;
    promo: Schema['StoryblokDraftBlockScalar'] | null;
    sponsored_content_target: string | null;
    syndication_id: string | null;
    table_of_contents_type: string | null;
    tags?: Array<Schema['StoryblokDraftStory'] | null> | null;
    updated_date: string | null;
  };
  'StoryblokDraftEditorialnewsarticleFilterQuery': {
    author: Schema['StoryblokDraftFilterQueryOperations'] | null;
    gpt_ad_overide: Schema['StoryblokDraftFilterQueryOperations'] | null;
    is_featured_content_enabled: Schema['StoryblokDraftFilterQueryOperations'] | null;
    table_of_contents_type: Schema['StoryblokDraftFilterQueryOperations'] | null;
    primary_area: Schema['StoryblokDraftFilterQueryOperations'] | null;
    sponsored_content_target: Schema['StoryblokDraftFilterQueryOperations'] | null;
    is_negative: Schema['StoryblokDraftFilterQueryOperations'] | null;
    disable_on_pos: Schema['StoryblokDraftFilterQueryOperations'] | null;
    tags: Schema['StoryblokDraftFilterQueryOperations'] | null;
    area_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    syndication_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    external_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    client_key: Schema['StoryblokDraftFilterQueryOperations'] | null;
    is_no_index: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftEditorialnewsarticleItem': {
    __typename?: 'StoryblokDraftEditorialnewsarticleItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorialnewsarticleComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorialnewsarticleItems': {
    __typename?: 'StoryblokDraftEditorialnewsarticleItems';
    items?: Array<Schema['StoryblokDraftEditorialnewsarticleItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorpicksComponent': {
    __typename?: 'StoryblokDraftEditorpicksComponent';
    _editable: string | null;
    _uid: string | null;
    awards_image?: Schema['StoryblokDraftAsset'] | null;
    component: string | null;
    image?: Schema['StoryblokDraftAsset'] | null;
    intro: Schema['StoryblokDraftBlockScalar'] | null;
    title: string | null;
    winners: Schema['StoryblokDraftBlockScalar'] | null;
  };
  'StoryblokDraftEditorpicksFilterQuery': {
    title: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftEditorpicksItem': {
    __typename?: 'StoryblokDraftEditorpicksItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorpicksComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorpicksItems': {
    __typename?: 'StoryblokDraftEditorpicksItems';
    items?: Array<Schema['StoryblokDraftEditorpicksItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorpicksintroComponent': {
    __typename?: 'StoryblokDraftEditorpicksintroComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    description: string | null;
    title: string | null;
  };
  'StoryblokDraftEditorpicksintroFilterQuery': {
    title: Schema['StoryblokDraftFilterQueryOperations'] | null;
    description: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftEditorpicksintroItem': {
    __typename?: 'StoryblokDraftEditorpicksintroItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorpicksintroComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorpicksintroItems': {
    __typename?: 'StoryblokDraftEditorpicksintroItems';
    items?: Array<Schema['StoryblokDraftEditorpicksintroItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftEditorpickswinnerComponent': {
    __typename?: 'StoryblokDraftEditorpickswinnerComponent';
    _editable: string | null;
    _uid: string | null;
    category?: Schema['StoryblokDraftStory'] | null;
    component: string | null;
    description: string | null;
    find_a_cruise_link_title: string | null;
    hide: boolean | null;
    image?: Schema['StoryblokDraftAsset'] | null;
    name: string | null;
    reviews_link_title: string | null;
    subject?: Schema['StoryblokDraftStory'] | null;
  };
  'StoryblokDraftEditorpickswinnerFilterQuery': {
    name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    category: Schema['StoryblokDraftFilterQueryOperations'] | null;
    subject: Schema['StoryblokDraftFilterQueryOperations'] | null;
    find_a_cruise_link_title: Schema['StoryblokDraftFilterQueryOperations'] | null;
    reviews_link_title: Schema['StoryblokDraftFilterQueryOperations'] | null;
    hide: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftEditorpickswinnerItem': {
    __typename?: 'StoryblokDraftEditorpickswinnerItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftEditorpickswinnerComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftEditorpickswinnerItems': {
    __typename?: 'StoryblokDraftEditorpickswinnerItems';
    items?: Array<Schema['StoryblokDraftEditorpickswinnerItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftFaccruiseshipoverviewComponent': {
    __typename?: 'StoryblokDraftFaccruiseshipoverviewComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    hero_image: Schema['StoryblokDraftBlockScalar'] | null;
    intro_body: Schema['StoryblokDraftJsonScalar'] | null;
    intro_heading: string | null;
    ship?: Schema['StoryblokDraftStory'] | null;
  };
  'StoryblokDraftFaccruiseshipoverviewFilterQuery': {
    ship: Schema['StoryblokDraftFilterQueryOperations'] | null;
    intro_heading: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftFaccruiseshipoverviewItem': {
    __typename?: 'StoryblokDraftFaccruiseshipoverviewItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftFaccruiseshipoverviewComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftFaccruiseshipoverviewItems': {
    __typename?: 'StoryblokDraftFaccruiseshipoverviewItems';
    items?: Array<Schema['StoryblokDraftFaccruiseshipoverviewItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftFilterQueryOperations': {
  /**
   * Matches exactly one value
   */
    in: string | null;
  /**
   * Matches all without the given value
   */
    not_in: string | null;
  /**
   * Matches exactly one value with a wildcard search using *
   */
    like: string | null;
  /**
   * Matches all without the given value
   */
    not_like: string | null;
  /**
   * Matches any value of given array
   */
    in_array: Array<string | null> | null;
  /**
   * Must match all values of given array
   */
    all_in_array: Array<string | null> | null;
  /**
   * Greater than date (Exmples: 2019-03-03 or 2020-03-03T03:03:03)
   */
    gt_date: Schema['StoryblokDraftISO8601DateTime'] | null;
  /**
   * Less than date (Format: 2019-03-03 or 2020-03-03T03:03:03)
   */
    lt_date: Schema['StoryblokDraftISO8601DateTime'] | null;
  /**
   * Greater than integer value
   */
    gt_int: number | null;
  /**
   * Less than integer value
   */
    lt_int: number | null;
  /**
   * Matches exactly one integer value
   */
    in_int: number | null;
  /**
   * Greater than float value
   */
    gt_float: number | null;
  /**
   * Less than float value
   */
    lt_float: number | null;
  };
  'StoryblokDraftFirsttimecruiserComponent': {
    __typename?: 'StoryblokDraftFirsttimecruiserComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    external_id: string | null;
    image?: Schema['StoryblokDraftAsset'] | null;
    promo: string | null;
    sort_order: string | null;
    title: string | null;
  };
  'StoryblokDraftFirsttimecruiserFilterQuery': {
    external_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    sort_order: Schema['StoryblokDraftFilterQueryOperations'] | null;
    title: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftFirsttimecruiserItem': {
    __typename?: 'StoryblokDraftFirsttimecruiserItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftFirsttimecruiserComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftFirsttimecruiserItems': {
    __typename?: 'StoryblokDraftFirsttimecruiserItems';
    items?: Array<Schema['StoryblokDraftFirsttimecruiserItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftGoogleadComponent': {
    __typename?: 'StoryblokDraftGoogleadComponent';
    _editable: string | null;
    _uid: string | null;
    ad_type: string | null;
    component: string | null;
  };
  'StoryblokDraftGoogleadFilterQuery': {
    ad_type: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftGoogleadItem': {
    __typename?: 'StoryblokDraftGoogleadItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftGoogleadComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftGoogleadItems': {
    __typename?: 'StoryblokDraftGoogleadItems';
    items?: Array<Schema['StoryblokDraftGoogleadItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftHiddenvendorComponent': {
    __typename?: 'StoryblokDraftHiddenvendorComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    cruise_line?: Array<Schema['StoryblokDraftStory'] | null> | null;
    vendor?: Array<Schema['StoryblokDraftStory'] | null> | null;
  };
  'StoryblokDraftHiddenvendorFilterQuery': {
    cruise_line: Schema['StoryblokDraftFilterQueryOperations'] | null;
    vendor: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftHiddenvendorItem': {
    __typename?: 'StoryblokDraftHiddenvendorItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftHiddenvendorComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftHiddenvendorItems': {
    __typename?: 'StoryblokDraftHiddenvendorItems';
    items?: Array<Schema['StoryblokDraftHiddenvendorItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftHomepageComponent': {
    __typename?: 'StoryblokDraftHomepageComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    contents: Schema['StoryblokDraftBlockScalar'] | null;
    seo: Schema['StoryblokDraftJsonScalar'] | null;
  };
  'StoryblokDraftHomepageItem': {
    __typename?: 'StoryblokDraftHomepageItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftHomepageComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftHomepageItems': {
    __typename?: 'StoryblokDraftHomepageItems';
    items?: Array<Schema['StoryblokDraftHomepageItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftHorizonadComponent': {
    __typename?: 'StoryblokDraftHorizonadComponent';
    _editable: string | null;
    _uid: string | null;
    active: boolean | null;
    background_color: Schema['StoryblokDraftJsonScalar'] | null;
    button_border_color: Schema['StoryblokDraftJsonScalar'] | null;
    button_color: Schema['StoryblokDraftJsonScalar'] | null;
    button_text: string | null;
    button_text_color: Schema['StoryblokDraftJsonScalar'] | null;
    component: string | null;
    page: string | null;
    points_of_sale?: Array<Schema['StoryblokDraftStory'] | null> | null;
    promo: Schema['StoryblokDraftBlockScalar'] | null;
    secondary_text: string | null;
    text: string | null;
    title: string | null;
    tracking_pixel: string | null;
    url: string | null;
    vendor?: Array<Schema['StoryblokDraftStory'] | null> | null;
    vendor_logo: Schema['StoryblokDraftBlockScalar'] | null;
    vendor_secondary_logo: Schema['StoryblokDraftBlockScalar'] | null;
  };
  'StoryblokDraftHorizonadFilterQuery': {
    active: Schema['StoryblokDraftFilterQueryOperations'] | null;
    vendor: Schema['StoryblokDraftFilterQueryOperations'] | null;
    points_of_sale: Schema['StoryblokDraftFilterQueryOperations'] | null;
    button_text: Schema['StoryblokDraftFilterQueryOperations'] | null;
    text: Schema['StoryblokDraftFilterQueryOperations'] | null;
    secondary_text: Schema['StoryblokDraftFilterQueryOperations'] | null;
    page: Schema['StoryblokDraftFilterQueryOperations'] | null;
    tracking_pixel: Schema['StoryblokDraftFilterQueryOperations'] | null;
    url: Schema['StoryblokDraftFilterQueryOperations'] | null;
    title: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftHorizonadItem': {
    __typename?: 'StoryblokDraftHorizonadItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftHorizonadComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftHorizonadItems': {
    __typename?: 'StoryblokDraftHorizonadItems';
    items?: Array<Schema['StoryblokDraftHorizonadItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftHubpageComponent': {
    __typename?: 'StoryblokDraftHubpageComponent';
    _editable: string | null;
    _uid: string | null;
    body_content: Schema['StoryblokDraftBlockScalar'] | null;
    component: string | null;
    cruise_line?: Schema['StoryblokDraftStory'] | null;
    cruise_style?: Schema['StoryblokDraftStory'] | null;
    destination?: Schema['StoryblokDraftStory'] | null;
    gpt_ad_overide?: Schema['StoryblokDraftStory'] | null;
    hero: Schema['StoryblokDraftBlockScalar'] | null;
    seo: Schema['StoryblokDraftJsonScalar'] | null;
    ship?: Schema['StoryblokDraftStory'] | null;
    tags?: Array<Schema['StoryblokDraftStory'] | null> | null;
  };
  'StoryblokDraftHubpageFilterQuery': {
    destination: Schema['StoryblokDraftFilterQueryOperations'] | null;
    gpt_ad_overide: Schema['StoryblokDraftFilterQueryOperations'] | null;
    tags: Schema['StoryblokDraftFilterQueryOperations'] | null;
    cruise_line: Schema['StoryblokDraftFilterQueryOperations'] | null;
    ship: Schema['StoryblokDraftFilterQueryOperations'] | null;
    cruise_style: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftHubpageItem': {
    __typename?: 'StoryblokDraftHubpageItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftHubpageComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftHubpageItems': {
    __typename?: 'StoryblokDraftHubpageItems';
    items?: Array<Schema['StoryblokDraftHubpageItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftHubriverpageComponent': {
    __typename?: 'StoryblokDraftHubriverpageComponent';
    _editable: string | null;
    _uid: string | null;
    body_content: Schema['StoryblokDraftBlockScalar'] | null;
    component: string | null;
    destination?: Schema['StoryblokDraftStory'] | null;
    gpt_ad_overide?: Schema['StoryblokDraftStory'] | null;
    hero: Schema['StoryblokDraftBlockScalar'] | null;
    river_destination: string | null;
    seo: Schema['StoryblokDraftJsonScalar'] | null;
    tags?: Array<Schema['StoryblokDraftStory'] | null> | null;
  };
  'StoryblokDraftHubriverpageFilterQuery': {
    destination: Schema['StoryblokDraftFilterQueryOperations'] | null;
    river_destination: Schema['StoryblokDraftFilterQueryOperations'] | null;
    gpt_ad_overide: Schema['StoryblokDraftFilterQueryOperations'] | null;
    tags: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftHubriverpageItem': {
    __typename?: 'StoryblokDraftHubriverpageItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftHubriverpageComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftHubriverpageItems': {
    __typename?: 'StoryblokDraftHubriverpageItems';
    items?: Array<Schema['StoryblokDraftHubriverpageItem'] | null> | null;
    total: number | null;
  };
  /**
   * An ISO 8601-encoded datetime
   */
  'StoryblokDraftISO8601DateTime': any;
  'StoryblokDraftJsonScalar': any;
  'StoryblokDraftLanderapgedemoComponent': {
    __typename?: 'StoryblokDraftLanderapgedemoComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
  };
  'StoryblokDraftLanderapgedemoItem': {
    __typename?: 'StoryblokDraftLanderapgedemoItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftLanderapgedemoComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftLanderapgedemoItems': {
    __typename?: 'StoryblokDraftLanderapgedemoItems';
    items?: Array<Schema['StoryblokDraftLanderapgedemoItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftLink': {
    __typename?: 'StoryblokDraftLink';
    cachedUrl: string;
    email: string | null;
    fieldtype: string;
    id: string;
    linktype: string;
    story?: Schema['StoryblokDraftStory'] | null;
    url: string;
  };
  'StoryblokDraftLinkEntries': {
    __typename?: 'StoryblokDraftLinkEntries';
    items?: Array<Schema['StoryblokDraftLinkEntry']>;
  };
  'StoryblokDraftLinkEntry': {
    __typename?: 'StoryblokDraftLinkEntry';
    id: number | null;
    isFolder: boolean | null;
    isStartpage: boolean | null;
    name: string | null;
    parentId: number | null;
    position: number | null;
    published: boolean | null;
    slug: string | null;
    uuid: string | null;
  };
  'StoryblokDraftNativeadComponent': {
    __typename?: 'StoryblokDraftNativeadComponent';
    _editable: string | null;
    _uid: string | null;
    body: string | null;
    button_text: string | null;
    component: string | null;
    cruiseline?: Schema['StoryblokDraftStory'] | null;
    heading: string | null;
    image?: Schema['StoryblokDraftAsset'] | null;
    layout: string | null;
    link?: Schema['StoryblokDraftLink'] | null;
    subheading: string | null;
    vendor?: Schema['StoryblokDraftStory'] | null;
  };
  'StoryblokDraftNativeadFilterQuery': {
    vendor: Schema['StoryblokDraftFilterQueryOperations'] | null;
    layout: Schema['StoryblokDraftFilterQueryOperations'] | null;
    cruiseline: Schema['StoryblokDraftFilterQueryOperations'] | null;
    heading: Schema['StoryblokDraftFilterQueryOperations'] | null;
    subheading: Schema['StoryblokDraftFilterQueryOperations'] | null;
    button_text: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftNativeadItem': {
    __typename?: 'StoryblokDraftNativeadItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftNativeadComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftNativeadItems': {
    __typename?: 'StoryblokDraftNativeadItems';
    items?: Array<Schema['StoryblokDraftNativeadItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftNativeadsmallComponent': {
    __typename?: 'StoryblokDraftNativeadsmallComponent';
    _editable: string | null;
    _uid: string | null;
    body: string | null;
    button_text: string | null;
    component: string | null;
    cruiseline?: Schema['StoryblokDraftStory'] | null;
    heading: string | null;
    image?: Schema['StoryblokDraftAsset'] | null;
    link?: Schema['StoryblokDraftLink'] | null;
    vendor?: Schema['StoryblokDraftStory'] | null;
  };
  'StoryblokDraftNativeadsmallFilterQuery': {
    vendor: Schema['StoryblokDraftFilterQueryOperations'] | null;
    cruiseline: Schema['StoryblokDraftFilterQueryOperations'] | null;
    heading: Schema['StoryblokDraftFilterQueryOperations'] | null;
    button_text: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftNativeadsmallItem': {
    __typename?: 'StoryblokDraftNativeadsmallItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftNativeadsmallComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftNativeadsmallItems': {
    __typename?: 'StoryblokDraftNativeadsmallItems';
    items?: Array<Schema['StoryblokDraftNativeadsmallItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftNavigationComponent': {
    __typename?: 'StoryblokDraftNavigationComponent';
    _editable: string | null;
    _uid: string | null;
    article?: Schema['StoryblokDraftStory'] | null;
    component: string | null;
    cruiseline?: Schema['StoryblokDraftStory'] | null;
    deal_old_price: string | null;
    deal_price: string | null;
    default_title: string | null;
    ship?: Schema['StoryblokDraftStory'] | null;
  };
  'StoryblokDraftNavigationFilterQuery': {
    ship: Schema['StoryblokDraftFilterQueryOperations'] | null;
    default_title: Schema['StoryblokDraftFilterQueryOperations'] | null;
    deal_old_price: Schema['StoryblokDraftFilterQueryOperations'] | null;
    deal_price: Schema['StoryblokDraftFilterQueryOperations'] | null;
    cruiseline: Schema['StoryblokDraftFilterQueryOperations'] | null;
    article: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftNavigationItem': {
    __typename?: 'StoryblokDraftNavigationItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftNavigationComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftNavigationItems': {
    __typename?: 'StoryblokDraftNavigationItems';
    items?: Array<Schema['StoryblokDraftNavigationItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftPointsofsaleComponent': {
    __typename?: 'StoryblokDraftPointsofsaleComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    default_currency: string | null;
    domain: string | null;
    id: string | null;
    locale: string | null;
  };
  'StoryblokDraftPointsofsaleFilterQuery': {
    id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    domain: Schema['StoryblokDraftFilterQueryOperations'] | null;
    locale: Schema['StoryblokDraftFilterQueryOperations'] | null;
    default_currency: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftPointsofsaleItem': {
    __typename?: 'StoryblokDraftPointsofsaleItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftPointsofsaleComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftPointsofsaleItems': {
    __typename?: 'StoryblokDraftPointsofsaleItems';
    items?: Array<Schema['StoryblokDraftPointsofsaleItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftPortsComponent': {
    __typename?: 'StoryblokDraftPortsComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    destination?: Schema['StoryblokDraftStory'] | null;
    external_id: string | null;
    forum_id: string | null;
    images: Schema['StoryblokDraftBlockScalar'] | null;
    is_private: boolean | null;
    is_river: boolean | null;
    latitude: string | null;
    longitude: string | null;
    name: string | null;
    sales_name: string | null;
    seo_name: string | null;
  };
  'StoryblokDraftPortsFilterQuery': {
    name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    sales_name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    seo_name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    destination: Schema['StoryblokDraftFilterQueryOperations'] | null;
    is_private: Schema['StoryblokDraftFilterQueryOperations'] | null;
    is_river: Schema['StoryblokDraftFilterQueryOperations'] | null;
    external_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    forum_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    latitude: Schema['StoryblokDraftFilterQueryOperations'] | null;
    longitude: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftPortsItem': {
    __typename?: 'StoryblokDraftPortsItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftPortsComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftPortsItems': {
    __typename?: 'StoryblokDraftPortsItems';
    items?: Array<Schema['StoryblokDraftPortsItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftQueryType': {
    __typename?: 'StoryblokDraftQueryType';
    AbtestItem?: Schema['StoryblokDraftAbtestItem'] | null;
    AbtestItems?: Schema['StoryblokDraftAbtestItems'] | null;
    ArticleheroItem?: Schema['StoryblokDraftArticleheroItem'] | null;
    ArticleheroItems?: Schema['StoryblokDraftArticleheroItems'] | null;
    ArticleweightedtagItem?: Schema['StoryblokDraftArticleweightedtagItem'] | null;
    ArticleweightedtagItems?: Schema['StoryblokDraftArticleweightedtagItems'] | null;
    ChoiseawardcategoryItem?: Schema['StoryblokDraftChoiseawardcategoryItem'] | null;
    ChoiseawardcategoryItems?: Schema['StoryblokDraftChoiseawardcategoryItems'] | null;
    ColorItem?: Schema['StoryblokDraftColorItem'] | null;
    ColorItems?: Schema['StoryblokDraftColorItems'] | null;
    ContentNode?: Schema['StoryblokDraftContentItem'] | null;
    ContentNodes?: Schema['StoryblokDraftContentItems'] | null;
    CruisehealthandsafetyItem?: Schema['StoryblokDraftCruisehealthandsafetyItem'] | null;
    CruisehealthandsafetyItems?: Schema['StoryblokDraftCruisehealthandsafetyItems'] | null;
    CruiselineItem?: Schema['StoryblokDraftCruiselineItem'] | null;
    CruiselineItems?: Schema['StoryblokDraftCruiselineItems'] | null;
    CruiserschoiceawardItem?: Schema['StoryblokDraftCruiserschoiceawardItem'] | null;
    CruiserschoiceawardItems?: Schema['StoryblokDraftCruiserschoiceawardItems'] | null;
    CruiseshipItem?: Schema['StoryblokDraftCruiseshipItem'] | null;
    CruiseshipItems?: Schema['StoryblokDraftCruiseshipItems'] | null;
    CruisestylesItem?: Schema['StoryblokDraftCruisestylesItem'] | null;
    CruisestylesItems?: Schema['StoryblokDraftCruisestylesItems'] | null;
    CustomheadlineItem?: Schema['StoryblokDraftCustomheadlineItem'] | null;
    CustomheadlineItems?: Schema['StoryblokDraftCustomheadlineItems'] | null;
    DatasourceEntries?: Schema['StoryblokDraftDatasourceEntries'] | null;
    Datasources?: Schema['StoryblokDraftDatasources'] | null;
    DepartureportoverviewItem?: Schema['StoryblokDraftDepartureportoverviewItem'] | null;
    DepartureportoverviewItems?: Schema['StoryblokDraftDepartureportoverviewItems'] | null;
    DepartureportsItem?: Schema['StoryblokDraftDepartureportsItem'] | null;
    DepartureportsItems?: Schema['StoryblokDraftDepartureportsItems'] | null;
    DestinationoverviewItem?: Schema['StoryblokDraftDestinationoverviewItem'] | null;
    DestinationoverviewItems?: Schema['StoryblokDraftDestinationoverviewItems'] | null;
    DestinationsItem?: Schema['StoryblokDraftDestinationsItem'] | null;
    DestinationsItems?: Schema['StoryblokDraftDestinationsItems'] | null;
    DestinationslItem?: Schema['StoryblokDraftDestinationslItem'] | null;
    DestinationslItems?: Schema['StoryblokDraftDestinationslItems'] | null;
    EditorialarticlehubItem?: Schema['StoryblokDraftEditorialarticlehubItem'] | null;
    EditorialarticlehubItems?: Schema['StoryblokDraftEditorialarticlehubItems'] | null;
    EditorialauthorsItem?: Schema['StoryblokDraftEditorialauthorsItem'] | null;
    EditorialauthorsItems?: Schema['StoryblokDraftEditorialauthorsItems'] | null;
    EditorialcontentItem?: Schema['StoryblokDraftEditorialcontentItem'] | null;
    EditorialcontentItems?: Schema['StoryblokDraftEditorialcontentItems'] | null;
    EditorialcruiselineoverviewItem?: Schema['StoryblokDraftEditorialcruiselineoverviewItem'] | null;
    EditorialcruiselineoverviewItems?: Schema['StoryblokDraftEditorialcruiselineoverviewItems'] | null;
    EditorialcruiseshipactivitiesItem?: Schema['StoryblokDraftEditorialcruiseshipactivitiesItem'] | null;
    EditorialcruiseshipactivitiesItems?: Schema['StoryblokDraftEditorialcruiseshipactivitiesItems'] | null;
    EditorialcruiseshipcabinItem?: Schema['StoryblokDraftEditorialcruiseshipcabinItem'] | null;
    EditorialcruiseshipcabinItems?: Schema['StoryblokDraftEditorialcruiseshipcabinItems'] | null;
    EditorialcruiseshipdeckplanItem?: Schema['StoryblokDraftEditorialcruiseshipdeckplanItem'] | null;
    EditorialcruiseshipdeckplanItems?: Schema['StoryblokDraftEditorialcruiseshipdeckplanItems'] | null;
    EditorialcruiseshipdiningItem?: Schema['StoryblokDraftEditorialcruiseshipdiningItem'] | null;
    EditorialcruiseshipdiningItems?: Schema['StoryblokDraftEditorialcruiseshipdiningItems'] | null;
    EditorialcruiseshipoverviewItem?: Schema['StoryblokDraftEditorialcruiseshipoverviewItem'] | null;
    EditorialcruiseshipoverviewItems?: Schema['StoryblokDraftEditorialcruiseshipoverviewItems'] | null;
    EditorialfeaturearticleItem?: Schema['StoryblokDraftEditorialfeaturearticleItem'] | null;
    EditorialfeaturearticleItems?: Schema['StoryblokDraftEditorialfeaturearticleItems'] | null;
    EditorialfeaturearticleheroItem?: Schema['StoryblokDraftEditorialfeaturearticleheroItem'] | null;
    EditorialfeaturearticleheroItems?: Schema['StoryblokDraftEditorialfeaturearticleheroItems'] | null;
    EditorialfeaturearticlelandingItem?: Schema['StoryblokDraftEditorialfeaturearticlelandingItem'] | null;
    EditorialfeaturearticlelandingItems?: Schema['StoryblokDraftEditorialfeaturearticlelandingItems'] | null;
    EditorialnewsarticleItem?: Schema['StoryblokDraftEditorialnewsarticleItem'] | null;
    EditorialnewsarticleItems?: Schema['StoryblokDraftEditorialnewsarticleItems'] | null;
    EditorpicksItem?: Schema['StoryblokDraftEditorpicksItem'] | null;
    EditorpicksItems?: Schema['StoryblokDraftEditorpicksItems'] | null;
    EditorpicksintroItem?: Schema['StoryblokDraftEditorpicksintroItem'] | null;
    EditorpicksintroItems?: Schema['StoryblokDraftEditorpicksintroItems'] | null;
    EditorpickswinnerItem?: Schema['StoryblokDraftEditorpickswinnerItem'] | null;
    EditorpickswinnerItems?: Schema['StoryblokDraftEditorpickswinnerItems'] | null;
    FaccruiseshipoverviewItem?: Schema['StoryblokDraftFaccruiseshipoverviewItem'] | null;
    FaccruiseshipoverviewItems?: Schema['StoryblokDraftFaccruiseshipoverviewItems'] | null;
    FirsttimecruiserItem?: Schema['StoryblokDraftFirsttimecruiserItem'] | null;
    FirsttimecruiserItems?: Schema['StoryblokDraftFirsttimecruiserItems'] | null;
    GoogleadItem?: Schema['StoryblokDraftGoogleadItem'] | null;
    GoogleadItems?: Schema['StoryblokDraftGoogleadItems'] | null;
    HiddenvendorItem?: Schema['StoryblokDraftHiddenvendorItem'] | null;
    HiddenvendorItems?: Schema['StoryblokDraftHiddenvendorItems'] | null;
    HomepageItem?: Schema['StoryblokDraftHomepageItem'] | null;
    HomepageItems?: Schema['StoryblokDraftHomepageItems'] | null;
    HorizonadItem?: Schema['StoryblokDraftHorizonadItem'] | null;
    HorizonadItems?: Schema['StoryblokDraftHorizonadItems'] | null;
    HubpageItem?: Schema['StoryblokDraftHubpageItem'] | null;
    HubpageItems?: Schema['StoryblokDraftHubpageItems'] | null;
    HubriverpageItem?: Schema['StoryblokDraftHubriverpageItem'] | null;
    HubriverpageItems?: Schema['StoryblokDraftHubriverpageItems'] | null;
    LanderapgedemoItem?: Schema['StoryblokDraftLanderapgedemoItem'] | null;
    LanderapgedemoItems?: Schema['StoryblokDraftLanderapgedemoItems'] | null;
    Links?: Schema['StoryblokDraftLinkEntries'] | null;
    NativeadItem?: Schema['StoryblokDraftNativeadItem'] | null;
    NativeadItems?: Schema['StoryblokDraftNativeadItems'] | null;
    NativeadsmallItem?: Schema['StoryblokDraftNativeadsmallItem'] | null;
    NativeadsmallItems?: Schema['StoryblokDraftNativeadsmallItems'] | null;
    NavigationItem?: Schema['StoryblokDraftNavigationItem'] | null;
    NavigationItems?: Schema['StoryblokDraftNavigationItems'] | null;
    PointsofsaleItem?: Schema['StoryblokDraftPointsofsaleItem'] | null;
    PointsofsaleItems?: Schema['StoryblokDraftPointsofsaleItems'] | null;
    PortsItem?: Schema['StoryblokDraftPortsItem'] | null;
    PortsItems?: Schema['StoryblokDraftPortsItems'] | null;
    QuestionanswerItem?: Schema['StoryblokDraftQuestionanswerItem'] | null;
    QuestionanswerItems?: Schema['StoryblokDraftQuestionanswerItems'] | null;
    QuizItem?: Schema['StoryblokDraftQuizItem'] | null;
    QuizItems?: Schema['StoryblokDraftQuizItems'] | null;
    RateLimit?: Schema['StoryblokDraftRateLimit'] | null;
    SalesvendorItem?: Schema['StoryblokDraftSalesvendorItem'] | null;
    SalesvendorItems?: Schema['StoryblokDraftSalesvendorItems'] | null;
    SeopageflagItem?: Schema['StoryblokDraftSeopageflagItem'] | null;
    SeopageflagItems?: Schema['StoryblokDraftSeopageflagItems'] | null;
    Space?: Schema['StoryblokDraftSpace'] | null;
    SponconsponsorItem?: Schema['StoryblokDraftSponconsponsorItem'] | null;
    SponconsponsorItems?: Schema['StoryblokDraftSponconsponsorItems'] | null;
    SponsoredcontentItem?: Schema['StoryblokDraftSponsoredcontentItem'] | null;
    SponsoredcontentItems?: Schema['StoryblokDraftSponsoredcontentItems'] | null;
    TableItem?: Schema['StoryblokDraftTableItem'] | null;
    TableItems?: Schema['StoryblokDraftTableItems'] | null;
    Tags?: Schema['StoryblokDraftTags'] | null;
  };
  'StoryblokDraftQuestionanswerComponent': {
    __typename?: 'StoryblokDraftQuestionanswerComponent';
    _editable: string | null;
    _uid: string | null;
    analytics_target: string | null;
    answer: Schema['StoryblokDraftJsonScalar'] | null;
    component: string | null;
    question: string | null;
  };
  'StoryblokDraftQuestionanswerFilterQuery': {
    question: Schema['StoryblokDraftFilterQueryOperations'] | null;
    analytics_target: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftQuestionanswerItem': {
    __typename?: 'StoryblokDraftQuestionanswerItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftQuestionanswerComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftQuestionanswerItems': {
    __typename?: 'StoryblokDraftQuestionanswerItems';
    items?: Array<Schema['StoryblokDraftQuestionanswerItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftQuizComponent': {
    __typename?: 'StoryblokDraftQuizComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    questions: Schema['StoryblokDraftBlockScalar'] | null;
  };
  'StoryblokDraftQuizItem': {
    __typename?: 'StoryblokDraftQuizItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftQuizComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftQuizItems': {
    __typename?: 'StoryblokDraftQuizItems';
    items?: Array<Schema['StoryblokDraftQuizItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftRateLimit': {
    __typename?: 'StoryblokDraftRateLimit';
    maxCost: number;
  };
  'StoryblokDraftSalesvendorComponent': {
    __typename?: 'StoryblokDraftSalesvendorComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    id: string | null;
    name: string | null;
  };
  'StoryblokDraftSalesvendorFilterQuery': {
    id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    name: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftSalesvendorItem': {
    __typename?: 'StoryblokDraftSalesvendorItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftSalesvendorComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftSalesvendorItems': {
    __typename?: 'StoryblokDraftSalesvendorItems';
    items?: Array<Schema['StoryblokDraftSalesvendorItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftSeopageflagComponent': {
    __typename?: 'StoryblokDraftSeopageflagComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    description: string | null;
    h1: string | null;
    page_title: string | null;
  };
  'StoryblokDraftSeopageflagFilterQuery': {
    page_title: Schema['StoryblokDraftFilterQueryOperations'] | null;
    description: Schema['StoryblokDraftFilterQueryOperations'] | null;
    h1: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftSeopageflagItem': {
    __typename?: 'StoryblokDraftSeopageflagItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftSeopageflagComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftSeopageflagItems': {
    __typename?: 'StoryblokDraftSeopageflagItems';
    items?: Array<Schema['StoryblokDraftSeopageflagItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftSpace': {
    __typename?: 'StoryblokDraftSpace';
    domain: string;
    id: number;
    languageCodes: Array<string | null>;
    name: string;
    version: number;
  };
  'StoryblokDraftSponconsponsorComponent': {
    __typename?: 'StoryblokDraftSponconsponsorComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    description: string | null;
    facebook?: Schema['StoryblokDraftLink'] | null;
    instagram?: Schema['StoryblokDraftLink'] | null;
    logo?: Schema['StoryblokDraftAsset'] | null;
    name: string | null;
    pinterest?: Schema['StoryblokDraftLink'] | null;
    twitter?: Schema['StoryblokDraftLink'] | null;
    visit_url?: Schema['StoryblokDraftLink'] | null;
    youtube?: Schema['StoryblokDraftLink'] | null;
  };
  'StoryblokDraftSponconsponsorFilterQuery': {
    name: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftSponconsponsorItem': {
    __typename?: 'StoryblokDraftSponconsponsorItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftSponconsponsorComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftSponconsponsorItems': {
    __typename?: 'StoryblokDraftSponconsponsorItems';
    items?: Array<Schema['StoryblokDraftSponconsponsorItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftSponsoredcontentComponent': {
    __typename?: 'StoryblokDraftSponsoredcontentComponent';
    _editable: string | null;
    _uid: string | null;
    adobe_page_name: string | null;
    articles: Schema['StoryblokDraftBlockScalar'] | null;
    component: string | null;
    digioh_component_id: string | null;
    enable_digioh_signup: boolean | null;
    hero: Schema['StoryblokDraftBlockScalar'] | null;
    introduction: Schema['StoryblokDraftBlockScalar'] | null;
    is_no_index: boolean | null;
    metadata: Schema['StoryblokDraftJsonScalar'] | null;
    sponsor?: Schema['StoryblokDraftStory'] | null;
    tracking_click_tag: string | null;
    tracking_page_impression_tag: string | null;
  };
  'StoryblokDraftSponsoredcontentFilterQuery': {
    is_no_index: Schema['StoryblokDraftFilterQueryOperations'] | null;
    adobe_page_name: Schema['StoryblokDraftFilterQueryOperations'] | null;
    sponsor: Schema['StoryblokDraftFilterQueryOperations'] | null;
    enable_digioh_signup: Schema['StoryblokDraftFilterQueryOperations'] | null;
    digioh_component_id: Schema['StoryblokDraftFilterQueryOperations'] | null;
    tracking_click_tag: Schema['StoryblokDraftFilterQueryOperations'] | null;
    tracking_page_impression_tag: Schema['StoryblokDraftFilterQueryOperations'] | null;
  };
  'StoryblokDraftSponsoredcontentItem': {
    __typename?: 'StoryblokDraftSponsoredcontentItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftSponsoredcontentComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftSponsoredcontentItems': {
    __typename?: 'StoryblokDraftSponsoredcontentItems';
    items?: Array<Schema['StoryblokDraftSponsoredcontentItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftStory': {
    __typename?: 'StoryblokDraftStory';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content: Schema['StoryblokDraftJsonScalar'] | null;
    createdAt: string | null;
    firstPublishedAt: string | null;
    fullSlug: string | null;
    groupId: number | null;
    id: number | null;
    isStartpage: boolean | null;
    lang: string | null;
    metaData: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parentId: number | null;
    path: string | null;
    position: number | null;
    publishedAt: string | null;
    releaseId: number | null;
    slug: string | null;
    sortByDate: string | null;
    tagList: Array<string | null> | null;
    translatedSlugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftTableComponent': {
    __typename?: 'StoryblokDraftTableComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
  };
  'StoryblokDraftTableItem': {
    __typename?: 'StoryblokDraftTableItem';
    alternates?: Array<Schema['StoryblokDraftAlternate'] | null> | null;
    content?: Schema['StoryblokDraftTableComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokDraftJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokDraftTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokDraftTableItems': {
    __typename?: 'StoryblokDraftTableItems';
    items?: Array<Schema['StoryblokDraftTableItem'] | null> | null;
    total: number | null;
  };
  'StoryblokDraftTag': {
    __typename?: 'StoryblokDraftTag';
    name: string;
    taggingsCount: number;
  };
  'StoryblokDraftTags': {
    __typename?: 'StoryblokDraftTags';
    items?: Array<Schema['StoryblokDraftTag']>;
  };
  'StoryblokDraftTranslatedSlug': {
    __typename?: 'StoryblokDraftTranslatedSlug';
    lang: string;
    name: string | null;
    path: string | null;
  };
  'StoryblokEditorialarticlehubComponent': {
    __typename?: 'StoryblokEditorialarticlehubComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
  };
  'StoryblokEditorialarticlehubItem': {
    __typename?: 'StoryblokEditorialarticlehubItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorialarticlehubComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorialarticlehubItems': {
    __typename?: 'StoryblokEditorialarticlehubItems';
    items?: Array<Schema['StoryblokEditorialarticlehubItem'] | null> | null;
    total: number | null;
  };
  'StoryblokEditorialauthorsComponent': {
    __typename?: 'StoryblokEditorialauthorsComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    image?: Schema['StoryblokAsset'] | null;
    name: string | null;
    title: string | null;
  };
  'StoryblokEditorialauthorsFilterQuery': {
    name: Schema['StoryblokFilterQueryOperations'] | null;
    title: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokEditorialauthorsItem': {
    __typename?: 'StoryblokEditorialauthorsItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorialauthorsComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorialauthorsItems': {
    __typename?: 'StoryblokEditorialauthorsItems';
    items?: Array<Schema['StoryblokEditorialauthorsItem'] | null> | null;
    total: number | null;
  };
  'StoryblokEditorialcontentComponent': {
    __typename?: 'StoryblokEditorialcontentComponent';
    _editable: string | null;
    _uid: string | null;
    body: Schema['StoryblokJsonScalar'] | null;
    component: string | null;
  };
  'StoryblokEditorialcontentItem': {
    __typename?: 'StoryblokEditorialcontentItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorialcontentComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorialcontentItems': {
    __typename?: 'StoryblokEditorialcontentItems';
    items?: Array<Schema['StoryblokEditorialcontentItem'] | null> | null;
    total: number | null;
  };
  'StoryblokEditorialcruiselineoverviewComponent': {
    __typename?: 'StoryblokEditorialcruiselineoverviewComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    intro: Schema['StoryblokJsonScalar'] | null;
    partner_message: Schema['StoryblokBlockScalar'] | null;
    questions_answers: Schema['StoryblokBlockScalar'] | null;
  };
  'StoryblokEditorialcruiselineoverviewItem': {
    __typename?: 'StoryblokEditorialcruiselineoverviewItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorialcruiselineoverviewComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorialcruiselineoverviewItems': {
    __typename?: 'StoryblokEditorialcruiselineoverviewItems';
    items?: Array<Schema['StoryblokEditorialcruiselineoverviewItem'] | null> | null;
    total: number | null;
  };
  'StoryblokEditorialcruiseshipactivitiesComponent': {
    __typename?: 'StoryblokEditorialcruiseshipactivitiesComponent';
    _editable: string | null;
    _uid: string | null;
    activities_and_entertainment: Schema['StoryblokBlockScalar'] | null;
    author?: Schema['StoryblokStory'] | null;
    body: Schema['StoryblokBlockScalar'] | null;
    component: string | null;
    editorial_rating: Schema['StoryblokBlockScalar'] | null;
    header_assets: Schema['StoryblokBlockScalar'] | null;
    metatags: Schema['StoryblokJsonScalar'] | null;
    ship?: Schema['StoryblokStory'] | null;
  };
  'StoryblokEditorialcruiseshipactivitiesFilterQuery': {
    ship: Schema['StoryblokFilterQueryOperations'] | null;
    author: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokEditorialcruiseshipactivitiesItem': {
    __typename?: 'StoryblokEditorialcruiseshipactivitiesItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorialcruiseshipactivitiesComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorialcruiseshipactivitiesItems': {
    __typename?: 'StoryblokEditorialcruiseshipactivitiesItems';
    items?: Array<Schema['StoryblokEditorialcruiseshipactivitiesItem'] | null> | null;
    total: number | null;
  };
  'StoryblokEditorialcruiseshipcabinComponent': {
    __typename?: 'StoryblokEditorialcruiseshipcabinComponent';
    _editable: string | null;
    _uid: string | null;
    author?: Schema['StoryblokStory'] | null;
    category_assets: Schema['StoryblokBlockScalar'] | null;
    component: string | null;
    editorial_rating: Schema['StoryblokBlockScalar'] | null;
    header_assets: Schema['StoryblokBlockScalar'] | null;
    intro: Schema['StoryblokBlockScalar'] | null;
    metatags: Schema['StoryblokJsonScalar'] | null;
    ship?: Schema['StoryblokStory'] | null;
    text: Schema['StoryblokBlockScalar'] | null;
  };
  'StoryblokEditorialcruiseshipcabinFilterQuery': {
    ship: Schema['StoryblokFilterQueryOperations'] | null;
    author: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokEditorialcruiseshipcabinItem': {
    __typename?: 'StoryblokEditorialcruiseshipcabinItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorialcruiseshipcabinComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorialcruiseshipcabinItems': {
    __typename?: 'StoryblokEditorialcruiseshipcabinItems';
    items?: Array<Schema['StoryblokEditorialcruiseshipcabinItem'] | null> | null;
    total: number | null;
  };
  'StoryblokEditorialcruiseshipdeckplanComponent': {
    __typename?: 'StoryblokEditorialcruiseshipdeckplanComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    decks: Schema['StoryblokBlockScalar'] | null;
    metatags: Schema['StoryblokJsonScalar'] | null;
    ship?: Schema['StoryblokStory'] | null;
  };
  'StoryblokEditorialcruiseshipdeckplanFilterQuery': {
    ship: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokEditorialcruiseshipdeckplanItem': {
    __typename?: 'StoryblokEditorialcruiseshipdeckplanItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorialcruiseshipdeckplanComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorialcruiseshipdeckplanItems': {
    __typename?: 'StoryblokEditorialcruiseshipdeckplanItems';
    items?: Array<Schema['StoryblokEditorialcruiseshipdeckplanItem'] | null> | null;
    total: number | null;
  };
  'StoryblokEditorialcruiseshipdiningComponent': {
    __typename?: 'StoryblokEditorialcruiseshipdiningComponent';
    _editable: string | null;
    _uid: string | null;
    author?: Schema['StoryblokStory'] | null;
    body: Schema['StoryblokBlockScalar'] | null;
    component: string | null;
    editorial_rating: Schema['StoryblokBlockScalar'] | null;
    header_assets: Schema['StoryblokBlockScalar'] | null;
    metatags: Schema['StoryblokJsonScalar'] | null;
    restaurants: Schema['StoryblokBlockScalar'] | null;
    ship?: Schema['StoryblokStory'] | null;
  };
  'StoryblokEditorialcruiseshipdiningFilterQuery': {
    ship: Schema['StoryblokFilterQueryOperations'] | null;
    author: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokEditorialcruiseshipdiningItem': {
    __typename?: 'StoryblokEditorialcruiseshipdiningItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorialcruiseshipdiningComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorialcruiseshipdiningItems': {
    __typename?: 'StoryblokEditorialcruiseshipdiningItems';
    items?: Array<Schema['StoryblokEditorialcruiseshipdiningItem'] | null> | null;
    total: number | null;
  };
  'StoryblokEditorialcruiseshipoverviewComponent': {
    __typename?: 'StoryblokEditorialcruiseshipoverviewComponent';
    _editable: string | null;
    _uid: string | null;
    author?: Schema['StoryblokStory'] | null;
    component: string | null;
    dress_codes: Schema['StoryblokBlockScalar'] | null;
    editorial_rating: Schema['StoryblokBlockScalar'] | null;
    exclusions: Schema['StoryblokBlockScalar'] | null;
    exclusions_text: string | null;
    fellow_passengers: Schema['StoryblokBlockScalar'] | null;
    header_assets: Schema['StoryblokBlockScalar'] | null;
    inclusions: Schema['StoryblokBlockScalar'] | null;
    inclusions_text: string | null;
    intro: Schema['StoryblokBlockScalar'] | null;
    metatags: Schema['StoryblokJsonScalar'] | null;
    metatags_variation1: Schema['StoryblokJsonScalar'] | null;
    metatags_variation2: Schema['StoryblokJsonScalar'] | null;
    overview: Schema['StoryblokBlockScalar'] | null;
    review_highlights: Schema['StoryblokBlockScalar'] | null;
    ship?: Schema['StoryblokStory'] | null;
  };
  'StoryblokEditorialcruiseshipoverviewFilterQuery': {
    ship: Schema['StoryblokFilterQueryOperations'] | null;
    author: Schema['StoryblokFilterQueryOperations'] | null;
    inclusions_text: Schema['StoryblokFilterQueryOperations'] | null;
    exclusions_text: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokEditorialcruiseshipoverviewItem': {
    __typename?: 'StoryblokEditorialcruiseshipoverviewItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorialcruiseshipoverviewComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorialcruiseshipoverviewItems': {
    __typename?: 'StoryblokEditorialcruiseshipoverviewItems';
    items?: Array<Schema['StoryblokEditorialcruiseshipoverviewItem'] | null> | null;
    total: number | null;
  };
  'StoryblokEditorialfeaturearticleComponent': {
    __typename?: 'StoryblokEditorialfeaturearticleComponent';
    _editable: string | null;
    _uid: string | null;
    area_id?: Schema['StoryblokStory'] | null;
    author?: Array<Schema['StoryblokStory'] | null> | null;
    body: Schema['StoryblokBlockScalar'] | null;
    client_key: string | null;
    component: string | null;
    disable_on_pos: Array<string | null> | null;
    external_id: string | null;
    gpt_ad_overide?: Schema['StoryblokStory'] | null;
    headline: Schema['StoryblokBlockScalar'] | null;
    hero_image: Schema['StoryblokBlockScalar'] | null;
    is_featured_content_enabled: boolean | null;
    is_negative: boolean | null;
    is_no_index: boolean | null;
    keywords: string | null;
    metatags: Schema['StoryblokJsonScalar'] | null;
    primary_area: string | null;
    promo: Schema['StoryblokBlockScalar'] | null;
    sponsored_content_target: string | null;
    syndication_id: string | null;
    table_of_contents_type: string | null;
    tags?: Array<Schema['StoryblokStory'] | null> | null;
    updated_date: string | null;
  };
  'StoryblokEditorialfeaturearticleFilterQuery': {
    author: Schema['StoryblokFilterQueryOperations'] | null;
    gpt_ad_overide: Schema['StoryblokFilterQueryOperations'] | null;
    table_of_contents_type: Schema['StoryblokFilterQueryOperations'] | null;
    primary_area: Schema['StoryblokFilterQueryOperations'] | null;
    sponsored_content_target: Schema['StoryblokFilterQueryOperations'] | null;
    is_featured_content_enabled: Schema['StoryblokFilterQueryOperations'] | null;
    is_negative: Schema['StoryblokFilterQueryOperations'] | null;
    disable_on_pos: Schema['StoryblokFilterQueryOperations'] | null;
    tags: Schema['StoryblokFilterQueryOperations'] | null;
    area_id: Schema['StoryblokFilterQueryOperations'] | null;
    syndication_id: Schema['StoryblokFilterQueryOperations'] | null;
    external_id: Schema['StoryblokFilterQueryOperations'] | null;
    client_key: Schema['StoryblokFilterQueryOperations'] | null;
    is_no_index: Schema['StoryblokFilterQueryOperations'] | null;
    keywords: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokEditorialfeaturearticleItem': {
    __typename?: 'StoryblokEditorialfeaturearticleItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorialfeaturearticleComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorialfeaturearticleItems': {
    __typename?: 'StoryblokEditorialfeaturearticleItems';
    items?: Array<Schema['StoryblokEditorialfeaturearticleItem'] | null> | null;
    total: number | null;
  };
  'StoryblokEditorialfeaturearticleheroComponent': {
    __typename?: 'StoryblokEditorialfeaturearticleheroComponent';
    _editable: string | null;
    _uid: string | null;
    article?: Schema['StoryblokStory'] | null;
    component: string | null;
  };
  'StoryblokEditorialfeaturearticleheroFilterQuery': {
    article: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokEditorialfeaturearticleheroItem': {
    __typename?: 'StoryblokEditorialfeaturearticleheroItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorialfeaturearticleheroComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorialfeaturearticleheroItems': {
    __typename?: 'StoryblokEditorialfeaturearticleheroItems';
    items?: Array<Schema['StoryblokEditorialfeaturearticleheroItem'] | null> | null;
    total: number | null;
  };
  'StoryblokEditorialfeaturearticlelandingComponent': {
    __typename?: 'StoryblokEditorialfeaturearticlelandingComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    editors_picks_articles: Schema['StoryblokBlockScalar'] | null;
    hero_article: Schema['StoryblokBlockScalar'] | null;
  };
  'StoryblokEditorialfeaturearticlelandingItem': {
    __typename?: 'StoryblokEditorialfeaturearticlelandingItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorialfeaturearticlelandingComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorialfeaturearticlelandingItems': {
    __typename?: 'StoryblokEditorialfeaturearticlelandingItems';
    items?: Array<Schema['StoryblokEditorialfeaturearticlelandingItem'] | null> | null;
    total: number | null;
  };
  'StoryblokEditorialnewsarticleComponent': {
    __typename?: 'StoryblokEditorialnewsarticleComponent';
    _editable: string | null;
    _uid: string | null;
    area_id?: Schema['StoryblokStory'] | null;
    author?: Array<Schema['StoryblokStory'] | null> | null;
    body: Schema['StoryblokBlockScalar'] | null;
    client_key: string | null;
    component: string | null;
    disable_on_pos: Array<string | null> | null;
    external_id: string | null;
    gpt_ad_overide?: Schema['StoryblokStory'] | null;
    headline: Schema['StoryblokBlockScalar'] | null;
    hero_image: Schema['StoryblokBlockScalar'] | null;
    is_featured_content_enabled: boolean | null;
    is_negative: boolean | null;
    is_no_index: boolean | null;
    metatags: Schema['StoryblokJsonScalar'] | null;
    primary_area: string | null;
    promo: Schema['StoryblokBlockScalar'] | null;
    sponsored_content_target: string | null;
    syndication_id: string | null;
    table_of_contents_type: string | null;
    tags?: Array<Schema['StoryblokStory'] | null> | null;
    updated_date: string | null;
  };
  'StoryblokEditorialnewsarticleFilterQuery': {
    author: Schema['StoryblokFilterQueryOperations'] | null;
    gpt_ad_overide: Schema['StoryblokFilterQueryOperations'] | null;
    is_featured_content_enabled: Schema['StoryblokFilterQueryOperations'] | null;
    table_of_contents_type: Schema['StoryblokFilterQueryOperations'] | null;
    primary_area: Schema['StoryblokFilterQueryOperations'] | null;
    sponsored_content_target: Schema['StoryblokFilterQueryOperations'] | null;
    is_negative: Schema['StoryblokFilterQueryOperations'] | null;
    disable_on_pos: Schema['StoryblokFilterQueryOperations'] | null;
    tags: Schema['StoryblokFilterQueryOperations'] | null;
    area_id: Schema['StoryblokFilterQueryOperations'] | null;
    syndication_id: Schema['StoryblokFilterQueryOperations'] | null;
    external_id: Schema['StoryblokFilterQueryOperations'] | null;
    client_key: Schema['StoryblokFilterQueryOperations'] | null;
    is_no_index: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokEditorialnewsarticleItem': {
    __typename?: 'StoryblokEditorialnewsarticleItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorialnewsarticleComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorialnewsarticleItems': {
    __typename?: 'StoryblokEditorialnewsarticleItems';
    items?: Array<Schema['StoryblokEditorialnewsarticleItem'] | null> | null;
    total: number | null;
  };
  'StoryblokEditorpicksComponent': {
    __typename?: 'StoryblokEditorpicksComponent';
    _editable: string | null;
    _uid: string | null;
    awards_image?: Schema['StoryblokAsset'] | null;
    component: string | null;
    image?: Schema['StoryblokAsset'] | null;
    intro: Schema['StoryblokBlockScalar'] | null;
    title: string | null;
    winners: Schema['StoryblokBlockScalar'] | null;
  };
  'StoryblokEditorpicksFilterQuery': {
    title: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokEditorpicksItem': {
    __typename?: 'StoryblokEditorpicksItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorpicksComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorpicksItems': {
    __typename?: 'StoryblokEditorpicksItems';
    items?: Array<Schema['StoryblokEditorpicksItem'] | null> | null;
    total: number | null;
  };
  'StoryblokEditorpicksintroComponent': {
    __typename?: 'StoryblokEditorpicksintroComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    description: string | null;
    title: string | null;
  };
  'StoryblokEditorpicksintroFilterQuery': {
    title: Schema['StoryblokFilterQueryOperations'] | null;
    description: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokEditorpicksintroItem': {
    __typename?: 'StoryblokEditorpicksintroItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorpicksintroComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorpicksintroItems': {
    __typename?: 'StoryblokEditorpicksintroItems';
    items?: Array<Schema['StoryblokEditorpicksintroItem'] | null> | null;
    total: number | null;
  };
  'StoryblokEditorpickswinnerComponent': {
    __typename?: 'StoryblokEditorpickswinnerComponent';
    _editable: string | null;
    _uid: string | null;
    category?: Schema['StoryblokStory'] | null;
    component: string | null;
    description: string | null;
    find_a_cruise_link_title: string | null;
    hide: boolean | null;
    image?: Schema['StoryblokAsset'] | null;
    name: string | null;
    reviews_link_title: string | null;
    subject?: Schema['StoryblokStory'] | null;
  };
  'StoryblokEditorpickswinnerFilterQuery': {
    name: Schema['StoryblokFilterQueryOperations'] | null;
    category: Schema['StoryblokFilterQueryOperations'] | null;
    subject: Schema['StoryblokFilterQueryOperations'] | null;
    find_a_cruise_link_title: Schema['StoryblokFilterQueryOperations'] | null;
    reviews_link_title: Schema['StoryblokFilterQueryOperations'] | null;
    hide: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokEditorpickswinnerItem': {
    __typename?: 'StoryblokEditorpickswinnerItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokEditorpickswinnerComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokEditorpickswinnerItems': {
    __typename?: 'StoryblokEditorpickswinnerItems';
    items?: Array<Schema['StoryblokEditorpickswinnerItem'] | null> | null;
    total: number | null;
  };
  'StoryblokFaccruiseshipoverviewComponent': {
    __typename?: 'StoryblokFaccruiseshipoverviewComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    hero_image: Schema['StoryblokBlockScalar'] | null;
    intro_body: Schema['StoryblokJsonScalar'] | null;
    intro_heading: string | null;
    ship?: Schema['StoryblokStory'] | null;
  };
  'StoryblokFaccruiseshipoverviewFilterQuery': {
    ship: Schema['StoryblokFilterQueryOperations'] | null;
    intro_heading: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokFaccruiseshipoverviewItem': {
    __typename?: 'StoryblokFaccruiseshipoverviewItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokFaccruiseshipoverviewComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokFaccruiseshipoverviewItems': {
    __typename?: 'StoryblokFaccruiseshipoverviewItems';
    items?: Array<Schema['StoryblokFaccruiseshipoverviewItem'] | null> | null;
    total: number | null;
  };
  'StoryblokFilterQueryOperations': {
  /**
   * Matches exactly one value
   */
    in: string | null;
  /**
   * Matches all without the given value
   */
    not_in: string | null;
  /**
   * Matches exactly one value with a wildcard search using *
   */
    like: string | null;
  /**
   * Matches all without the given value
   */
    not_like: string | null;
  /**
   * Matches any value of given array
   */
    in_array: Array<string | null> | null;
  /**
   * Must match all values of given array
   */
    all_in_array: Array<string | null> | null;
  /**
   * Greater than date (Exmples: 2019-03-03 or 2020-03-03T03:03:03)
   */
    gt_date: Schema['StoryblokISO8601DateTime'] | null;
  /**
   * Less than date (Format: 2019-03-03 or 2020-03-03T03:03:03)
   */
    lt_date: Schema['StoryblokISO8601DateTime'] | null;
  /**
   * Greater than integer value
   */
    gt_int: number | null;
  /**
   * Less than integer value
   */
    lt_int: number | null;
  /**
   * Matches exactly one integer value
   */
    in_int: number | null;
  /**
   * Greater than float value
   */
    gt_float: number | null;
  /**
   * Less than float value
   */
    lt_float: number | null;
  };
  'StoryblokFirsttimecruiserComponent': {
    __typename?: 'StoryblokFirsttimecruiserComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    external_id: string | null;
    image?: Schema['StoryblokAsset'] | null;
    promo: string | null;
    sort_order: string | null;
    title: string | null;
  };
  'StoryblokFirsttimecruiserFilterQuery': {
    external_id: Schema['StoryblokFilterQueryOperations'] | null;
    sort_order: Schema['StoryblokFilterQueryOperations'] | null;
    title: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokFirsttimecruiserItem': {
    __typename?: 'StoryblokFirsttimecruiserItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokFirsttimecruiserComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokFirsttimecruiserItems': {
    __typename?: 'StoryblokFirsttimecruiserItems';
    items?: Array<Schema['StoryblokFirsttimecruiserItem'] | null> | null;
    total: number | null;
  };
  'StoryblokGoogleadComponent': {
    __typename?: 'StoryblokGoogleadComponent';
    _editable: string | null;
    _uid: string | null;
    ad_type: string | null;
    component: string | null;
  };
  'StoryblokGoogleadFilterQuery': {
    ad_type: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokGoogleadItem': {
    __typename?: 'StoryblokGoogleadItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokGoogleadComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokGoogleadItems': {
    __typename?: 'StoryblokGoogleadItems';
    items?: Array<Schema['StoryblokGoogleadItem'] | null> | null;
    total: number | null;
  };
  'StoryblokHiddenvendorComponent': {
    __typename?: 'StoryblokHiddenvendorComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    cruise_line?: Array<Schema['StoryblokStory'] | null> | null;
    vendor?: Array<Schema['StoryblokStory'] | null> | null;
  };
  'StoryblokHiddenvendorFilterQuery': {
    cruise_line: Schema['StoryblokFilterQueryOperations'] | null;
    vendor: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokHiddenvendorItem': {
    __typename?: 'StoryblokHiddenvendorItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokHiddenvendorComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokHiddenvendorItems': {
    __typename?: 'StoryblokHiddenvendorItems';
    items?: Array<Schema['StoryblokHiddenvendorItem'] | null> | null;
    total: number | null;
  };
  'StoryblokHomepageComponent': {
    __typename?: 'StoryblokHomepageComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    contents: Schema['StoryblokBlockScalar'] | null;
    seo: Schema['StoryblokJsonScalar'] | null;
  };
  'StoryblokHomepageItem': {
    __typename?: 'StoryblokHomepageItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokHomepageComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokHomepageItems': {
    __typename?: 'StoryblokHomepageItems';
    items?: Array<Schema['StoryblokHomepageItem'] | null> | null;
    total: number | null;
  };
  'StoryblokHorizonadComponent': {
    __typename?: 'StoryblokHorizonadComponent';
    _editable: string | null;
    _uid: string | null;
    active: boolean | null;
    background_color: Schema['StoryblokJsonScalar'] | null;
    button_border_color: Schema['StoryblokJsonScalar'] | null;
    button_color: Schema['StoryblokJsonScalar'] | null;
    button_text: string | null;
    button_text_color: Schema['StoryblokJsonScalar'] | null;
    component: string | null;
    page: string | null;
    points_of_sale?: Array<Schema['StoryblokStory'] | null> | null;
    promo: Schema['StoryblokBlockScalar'] | null;
    secondary_text: string | null;
    text: string | null;
    title: string | null;
    tracking_pixel: string | null;
    url: string | null;
    vendor?: Array<Schema['StoryblokStory'] | null> | null;
    vendor_logo: Schema['StoryblokBlockScalar'] | null;
    vendor_secondary_logo: Schema['StoryblokBlockScalar'] | null;
  };
  'StoryblokHorizonadFilterQuery': {
    active: Schema['StoryblokFilterQueryOperations'] | null;
    vendor: Schema['StoryblokFilterQueryOperations'] | null;
    points_of_sale: Schema['StoryblokFilterQueryOperations'] | null;
    button_text: Schema['StoryblokFilterQueryOperations'] | null;
    text: Schema['StoryblokFilterQueryOperations'] | null;
    secondary_text: Schema['StoryblokFilterQueryOperations'] | null;
    page: Schema['StoryblokFilterQueryOperations'] | null;
    tracking_pixel: Schema['StoryblokFilterQueryOperations'] | null;
    url: Schema['StoryblokFilterQueryOperations'] | null;
    title: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokHorizonadItem': {
    __typename?: 'StoryblokHorizonadItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokHorizonadComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokHorizonadItems': {
    __typename?: 'StoryblokHorizonadItems';
    items?: Array<Schema['StoryblokHorizonadItem'] | null> | null;
    total: number | null;
  };
  'StoryblokHubpageComponent': {
    __typename?: 'StoryblokHubpageComponent';
    _editable: string | null;
    _uid: string | null;
    body_content: Schema['StoryblokBlockScalar'] | null;
    component: string | null;
    cruise_line?: Schema['StoryblokStory'] | null;
    cruise_style?: Schema['StoryblokStory'] | null;
    destination?: Schema['StoryblokStory'] | null;
    gpt_ad_overide?: Schema['StoryblokStory'] | null;
    hero: Schema['StoryblokBlockScalar'] | null;
    seo: Schema['StoryblokJsonScalar'] | null;
    ship?: Schema['StoryblokStory'] | null;
    tags?: Array<Schema['StoryblokStory'] | null> | null;
  };
  'StoryblokHubpageFilterQuery': {
    destination: Schema['StoryblokFilterQueryOperations'] | null;
    gpt_ad_overide: Schema['StoryblokFilterQueryOperations'] | null;
    tags: Schema['StoryblokFilterQueryOperations'] | null;
    cruise_line: Schema['StoryblokFilterQueryOperations'] | null;
    ship: Schema['StoryblokFilterQueryOperations'] | null;
    cruise_style: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokHubpageItem': {
    __typename?: 'StoryblokHubpageItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokHubpageComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokHubpageItems': {
    __typename?: 'StoryblokHubpageItems';
    items?: Array<Schema['StoryblokHubpageItem'] | null> | null;
    total: number | null;
  };
  'StoryblokHubriverpageComponent': {
    __typename?: 'StoryblokHubriverpageComponent';
    _editable: string | null;
    _uid: string | null;
    body_content: Schema['StoryblokBlockScalar'] | null;
    component: string | null;
    destination?: Schema['StoryblokStory'] | null;
    gpt_ad_overide?: Schema['StoryblokStory'] | null;
    hero: Schema['StoryblokBlockScalar'] | null;
    river_destination: string | null;
    seo: Schema['StoryblokJsonScalar'] | null;
    tags?: Array<Schema['StoryblokStory'] | null> | null;
  };
  'StoryblokHubriverpageFilterQuery': {
    destination: Schema['StoryblokFilterQueryOperations'] | null;
    river_destination: Schema['StoryblokFilterQueryOperations'] | null;
    gpt_ad_overide: Schema['StoryblokFilterQueryOperations'] | null;
    tags: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokHubriverpageItem': {
    __typename?: 'StoryblokHubriverpageItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokHubriverpageComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokHubriverpageItems': {
    __typename?: 'StoryblokHubriverpageItems';
    items?: Array<Schema['StoryblokHubriverpageItem'] | null> | null;
    total: number | null;
  };
  /**
   * An ISO 8601-encoded datetime
   */
  'StoryblokISO8601DateTime': any;
  'StoryblokJsonScalar': any;
  'StoryblokLanderapgedemoComponent': {
    __typename?: 'StoryblokLanderapgedemoComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
  };
  'StoryblokLanderapgedemoItem': {
    __typename?: 'StoryblokLanderapgedemoItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokLanderapgedemoComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokLanderapgedemoItems': {
    __typename?: 'StoryblokLanderapgedemoItems';
    items?: Array<Schema['StoryblokLanderapgedemoItem'] | null> | null;
    total: number | null;
  };
  'StoryblokLink': {
    __typename?: 'StoryblokLink';
    cachedUrl: string;
    email: string | null;
    fieldtype: string;
    id: string;
    linktype: string;
    story?: Schema['StoryblokStory'] | null;
    url: string;
  };
  'StoryblokLinkEntries': {
    __typename?: 'StoryblokLinkEntries';
    items?: Array<Schema['StoryblokLinkEntry']>;
  };
  'StoryblokLinkEntry': {
    __typename?: 'StoryblokLinkEntry';
    id: number | null;
    isFolder: boolean | null;
    isStartpage: boolean | null;
    name: string | null;
    parentId: number | null;
    position: number | null;
    published: boolean | null;
    slug: string | null;
    uuid: string | null;
  };
  'StoryblokNativeadComponent': {
    __typename?: 'StoryblokNativeadComponent';
    _editable: string | null;
    _uid: string | null;
    body: string | null;
    button_text: string | null;
    component: string | null;
    cruiseline?: Schema['StoryblokStory'] | null;
    heading: string | null;
    image?: Schema['StoryblokAsset'] | null;
    layout: string | null;
    link?: Schema['StoryblokLink'] | null;
    subheading: string | null;
    vendor?: Schema['StoryblokStory'] | null;
  };
  'StoryblokNativeadFilterQuery': {
    vendor: Schema['StoryblokFilterQueryOperations'] | null;
    layout: Schema['StoryblokFilterQueryOperations'] | null;
    cruiseline: Schema['StoryblokFilterQueryOperations'] | null;
    heading: Schema['StoryblokFilterQueryOperations'] | null;
    subheading: Schema['StoryblokFilterQueryOperations'] | null;
    button_text: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokNativeadItem': {
    __typename?: 'StoryblokNativeadItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokNativeadComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokNativeadItems': {
    __typename?: 'StoryblokNativeadItems';
    items?: Array<Schema['StoryblokNativeadItem'] | null> | null;
    total: number | null;
  };
  'StoryblokNativeadsmallComponent': {
    __typename?: 'StoryblokNativeadsmallComponent';
    _editable: string | null;
    _uid: string | null;
    body: string | null;
    button_text: string | null;
    component: string | null;
    cruiseline?: Schema['StoryblokStory'] | null;
    heading: string | null;
    image?: Schema['StoryblokAsset'] | null;
    link?: Schema['StoryblokLink'] | null;
    vendor?: Schema['StoryblokStory'] | null;
  };
  'StoryblokNativeadsmallFilterQuery': {
    vendor: Schema['StoryblokFilterQueryOperations'] | null;
    cruiseline: Schema['StoryblokFilterQueryOperations'] | null;
    heading: Schema['StoryblokFilterQueryOperations'] | null;
    button_text: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokNativeadsmallItem': {
    __typename?: 'StoryblokNativeadsmallItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokNativeadsmallComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokNativeadsmallItems': {
    __typename?: 'StoryblokNativeadsmallItems';
    items?: Array<Schema['StoryblokNativeadsmallItem'] | null> | null;
    total: number | null;
  };
  'StoryblokNavigationComponent': {
    __typename?: 'StoryblokNavigationComponent';
    _editable: string | null;
    _uid: string | null;
    article?: Schema['StoryblokStory'] | null;
    component: string | null;
    cruiseline?: Schema['StoryblokStory'] | null;
    deal_old_price: string | null;
    deal_price: string | null;
    default_title: string | null;
    ship?: Schema['StoryblokStory'] | null;
  };
  'StoryblokNavigationFilterQuery': {
    ship: Schema['StoryblokFilterQueryOperations'] | null;
    default_title: Schema['StoryblokFilterQueryOperations'] | null;
    deal_old_price: Schema['StoryblokFilterQueryOperations'] | null;
    deal_price: Schema['StoryblokFilterQueryOperations'] | null;
    cruiseline: Schema['StoryblokFilterQueryOperations'] | null;
    article: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokNavigationItem': {
    __typename?: 'StoryblokNavigationItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokNavigationComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokNavigationItems': {
    __typename?: 'StoryblokNavigationItems';
    items?: Array<Schema['StoryblokNavigationItem'] | null> | null;
    total: number | null;
  };
  'StoryblokPointsofsaleComponent': {
    __typename?: 'StoryblokPointsofsaleComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    default_currency: string | null;
    domain: string | null;
    id: string | null;
    locale: string | null;
  };
  'StoryblokPointsofsaleFilterQuery': {
    id: Schema['StoryblokFilterQueryOperations'] | null;
    domain: Schema['StoryblokFilterQueryOperations'] | null;
    locale: Schema['StoryblokFilterQueryOperations'] | null;
    default_currency: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokPointsofsaleItem': {
    __typename?: 'StoryblokPointsofsaleItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokPointsofsaleComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokPointsofsaleItems': {
    __typename?: 'StoryblokPointsofsaleItems';
    items?: Array<Schema['StoryblokPointsofsaleItem'] | null> | null;
    total: number | null;
  };
  'StoryblokPortsComponent': {
    __typename?: 'StoryblokPortsComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    destination?: Schema['StoryblokStory'] | null;
    external_id: string | null;
    forum_id: string | null;
    images: Schema['StoryblokBlockScalar'] | null;
    is_private: boolean | null;
    is_river: boolean | null;
    latitude: string | null;
    longitude: string | null;
    name: string | null;
    sales_name: string | null;
    seo_name: string | null;
  };
  'StoryblokPortsFilterQuery': {
    name: Schema['StoryblokFilterQueryOperations'] | null;
    sales_name: Schema['StoryblokFilterQueryOperations'] | null;
    seo_name: Schema['StoryblokFilterQueryOperations'] | null;
    destination: Schema['StoryblokFilterQueryOperations'] | null;
    is_private: Schema['StoryblokFilterQueryOperations'] | null;
    is_river: Schema['StoryblokFilterQueryOperations'] | null;
    external_id: Schema['StoryblokFilterQueryOperations'] | null;
    forum_id: Schema['StoryblokFilterQueryOperations'] | null;
    latitude: Schema['StoryblokFilterQueryOperations'] | null;
    longitude: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokPortsItem': {
    __typename?: 'StoryblokPortsItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokPortsComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokPortsItems': {
    __typename?: 'StoryblokPortsItems';
    items?: Array<Schema['StoryblokPortsItem'] | null> | null;
    total: number | null;
  };
  'StoryblokQueryType': {
    __typename?: 'StoryblokQueryType';
    AbtestItem?: Schema['StoryblokAbtestItem'] | null;
    AbtestItems?: Schema['StoryblokAbtestItems'] | null;
    ArticleheroItem?: Schema['StoryblokArticleheroItem'] | null;
    ArticleheroItems?: Schema['StoryblokArticleheroItems'] | null;
    ArticleweightedtagItem?: Schema['StoryblokArticleweightedtagItem'] | null;
    ArticleweightedtagItems?: Schema['StoryblokArticleweightedtagItems'] | null;
    ChoiseawardcategoryItem?: Schema['StoryblokChoiseawardcategoryItem'] | null;
    ChoiseawardcategoryItems?: Schema['StoryblokChoiseawardcategoryItems'] | null;
    ColorItem?: Schema['StoryblokColorItem'] | null;
    ColorItems?: Schema['StoryblokColorItems'] | null;
    ContentNode?: Schema['StoryblokContentItem'] | null;
    ContentNodes?: Schema['StoryblokContentItems'] | null;
    CruisehealthandsafetyItem?: Schema['StoryblokCruisehealthandsafetyItem'] | null;
    CruisehealthandsafetyItems?: Schema['StoryblokCruisehealthandsafetyItems'] | null;
    CruiselineItem?: Schema['StoryblokCruiselineItem'] | null;
    CruiselineItems?: Schema['StoryblokCruiselineItems'] | null;
    CruiserschoiceawardItem?: Schema['StoryblokCruiserschoiceawardItem'] | null;
    CruiserschoiceawardItems?: Schema['StoryblokCruiserschoiceawardItems'] | null;
    CruiseshipItem?: Schema['StoryblokCruiseshipItem'] | null;
    CruiseshipItems?: Schema['StoryblokCruiseshipItems'] | null;
    CruisestylesItem?: Schema['StoryblokCruisestylesItem'] | null;
    CruisestylesItems?: Schema['StoryblokCruisestylesItems'] | null;
    CustomheadlineItem?: Schema['StoryblokCustomheadlineItem'] | null;
    CustomheadlineItems?: Schema['StoryblokCustomheadlineItems'] | null;
    DatasourceEntries?: Schema['StoryblokDatasourceEntries'] | null;
    Datasources?: Schema['StoryblokDatasources'] | null;
    DepartureportoverviewItem?: Schema['StoryblokDepartureportoverviewItem'] | null;
    DepartureportoverviewItems?: Schema['StoryblokDepartureportoverviewItems'] | null;
    DepartureportsItem?: Schema['StoryblokDepartureportsItem'] | null;
    DepartureportsItems?: Schema['StoryblokDepartureportsItems'] | null;
    DestinationoverviewItem?: Schema['StoryblokDestinationoverviewItem'] | null;
    DestinationoverviewItems?: Schema['StoryblokDestinationoverviewItems'] | null;
    DestinationsItem?: Schema['StoryblokDestinationsItem'] | null;
    DestinationsItems?: Schema['StoryblokDestinationsItems'] | null;
    DestinationslItem?: Schema['StoryblokDestinationslItem'] | null;
    DestinationslItems?: Schema['StoryblokDestinationslItems'] | null;
    EditorialarticlehubItem?: Schema['StoryblokEditorialarticlehubItem'] | null;
    EditorialarticlehubItems?: Schema['StoryblokEditorialarticlehubItems'] | null;
    EditorialauthorsItem?: Schema['StoryblokEditorialauthorsItem'] | null;
    EditorialauthorsItems?: Schema['StoryblokEditorialauthorsItems'] | null;
    EditorialcontentItem?: Schema['StoryblokEditorialcontentItem'] | null;
    EditorialcontentItems?: Schema['StoryblokEditorialcontentItems'] | null;
    EditorialcruiselineoverviewItem?: Schema['StoryblokEditorialcruiselineoverviewItem'] | null;
    EditorialcruiselineoverviewItems?: Schema['StoryblokEditorialcruiselineoverviewItems'] | null;
    EditorialcruiseshipactivitiesItem?: Schema['StoryblokEditorialcruiseshipactivitiesItem'] | null;
    EditorialcruiseshipactivitiesItems?: Schema['StoryblokEditorialcruiseshipactivitiesItems'] | null;
    EditorialcruiseshipcabinItem?: Schema['StoryblokEditorialcruiseshipcabinItem'] | null;
    EditorialcruiseshipcabinItems?: Schema['StoryblokEditorialcruiseshipcabinItems'] | null;
    EditorialcruiseshipdeckplanItem?: Schema['StoryblokEditorialcruiseshipdeckplanItem'] | null;
    EditorialcruiseshipdeckplanItems?: Schema['StoryblokEditorialcruiseshipdeckplanItems'] | null;
    EditorialcruiseshipdiningItem?: Schema['StoryblokEditorialcruiseshipdiningItem'] | null;
    EditorialcruiseshipdiningItems?: Schema['StoryblokEditorialcruiseshipdiningItems'] | null;
    EditorialcruiseshipoverviewItem?: Schema['StoryblokEditorialcruiseshipoverviewItem'] | null;
    EditorialcruiseshipoverviewItems?: Schema['StoryblokEditorialcruiseshipoverviewItems'] | null;
    EditorialfeaturearticleItem?: Schema['StoryblokEditorialfeaturearticleItem'] | null;
    EditorialfeaturearticleItems?: Schema['StoryblokEditorialfeaturearticleItems'] | null;
    EditorialfeaturearticleheroItem?: Schema['StoryblokEditorialfeaturearticleheroItem'] | null;
    EditorialfeaturearticleheroItems?: Schema['StoryblokEditorialfeaturearticleheroItems'] | null;
    EditorialfeaturearticlelandingItem?: Schema['StoryblokEditorialfeaturearticlelandingItem'] | null;
    EditorialfeaturearticlelandingItems?: Schema['StoryblokEditorialfeaturearticlelandingItems'] | null;
    EditorialnewsarticleItem?: Schema['StoryblokEditorialnewsarticleItem'] | null;
    EditorialnewsarticleItems?: Schema['StoryblokEditorialnewsarticleItems'] | null;
    EditorpicksItem?: Schema['StoryblokEditorpicksItem'] | null;
    EditorpicksItems?: Schema['StoryblokEditorpicksItems'] | null;
    EditorpicksintroItem?: Schema['StoryblokEditorpicksintroItem'] | null;
    EditorpicksintroItems?: Schema['StoryblokEditorpicksintroItems'] | null;
    EditorpickswinnerItem?: Schema['StoryblokEditorpickswinnerItem'] | null;
    EditorpickswinnerItems?: Schema['StoryblokEditorpickswinnerItems'] | null;
    FaccruiseshipoverviewItem?: Schema['StoryblokFaccruiseshipoverviewItem'] | null;
    FaccruiseshipoverviewItems?: Schema['StoryblokFaccruiseshipoverviewItems'] | null;
    FirsttimecruiserItem?: Schema['StoryblokFirsttimecruiserItem'] | null;
    FirsttimecruiserItems?: Schema['StoryblokFirsttimecruiserItems'] | null;
    GoogleadItem?: Schema['StoryblokGoogleadItem'] | null;
    GoogleadItems?: Schema['StoryblokGoogleadItems'] | null;
    HiddenvendorItem?: Schema['StoryblokHiddenvendorItem'] | null;
    HiddenvendorItems?: Schema['StoryblokHiddenvendorItems'] | null;
    HomepageItem?: Schema['StoryblokHomepageItem'] | null;
    HomepageItems?: Schema['StoryblokHomepageItems'] | null;
    HorizonadItem?: Schema['StoryblokHorizonadItem'] | null;
    HorizonadItems?: Schema['StoryblokHorizonadItems'] | null;
    HubpageItem?: Schema['StoryblokHubpageItem'] | null;
    HubpageItems?: Schema['StoryblokHubpageItems'] | null;
    HubriverpageItem?: Schema['StoryblokHubriverpageItem'] | null;
    HubriverpageItems?: Schema['StoryblokHubriverpageItems'] | null;
    LanderapgedemoItem?: Schema['StoryblokLanderapgedemoItem'] | null;
    LanderapgedemoItems?: Schema['StoryblokLanderapgedemoItems'] | null;
    Links?: Schema['StoryblokLinkEntries'] | null;
    NativeadItem?: Schema['StoryblokNativeadItem'] | null;
    NativeadItems?: Schema['StoryblokNativeadItems'] | null;
    NativeadsmallItem?: Schema['StoryblokNativeadsmallItem'] | null;
    NativeadsmallItems?: Schema['StoryblokNativeadsmallItems'] | null;
    NavigationItem?: Schema['StoryblokNavigationItem'] | null;
    NavigationItems?: Schema['StoryblokNavigationItems'] | null;
    PointsofsaleItem?: Schema['StoryblokPointsofsaleItem'] | null;
    PointsofsaleItems?: Schema['StoryblokPointsofsaleItems'] | null;
    PortsItem?: Schema['StoryblokPortsItem'] | null;
    PortsItems?: Schema['StoryblokPortsItems'] | null;
    QuestionanswerItem?: Schema['StoryblokQuestionanswerItem'] | null;
    QuestionanswerItems?: Schema['StoryblokQuestionanswerItems'] | null;
    QuizItem?: Schema['StoryblokQuizItem'] | null;
    QuizItems?: Schema['StoryblokQuizItems'] | null;
    RateLimit?: Schema['StoryblokRateLimit'] | null;
    SalesvendorItem?: Schema['StoryblokSalesvendorItem'] | null;
    SalesvendorItems?: Schema['StoryblokSalesvendorItems'] | null;
    SeopageflagItem?: Schema['StoryblokSeopageflagItem'] | null;
    SeopageflagItems?: Schema['StoryblokSeopageflagItems'] | null;
    Space?: Schema['StoryblokSpace'] | null;
    SponconsponsorItem?: Schema['StoryblokSponconsponsorItem'] | null;
    SponconsponsorItems?: Schema['StoryblokSponconsponsorItems'] | null;
    SponsoredcontentItem?: Schema['StoryblokSponsoredcontentItem'] | null;
    SponsoredcontentItems?: Schema['StoryblokSponsoredcontentItems'] | null;
    TableItem?: Schema['StoryblokTableItem'] | null;
    TableItems?: Schema['StoryblokTableItems'] | null;
    Tags?: Schema['StoryblokTags'] | null;
  };
  'StoryblokQuestionanswerComponent': {
    __typename?: 'StoryblokQuestionanswerComponent';
    _editable: string | null;
    _uid: string | null;
    analytics_target: string | null;
    answer: Schema['StoryblokJsonScalar'] | null;
    component: string | null;
    question: string | null;
  };
  'StoryblokQuestionanswerFilterQuery': {
    question: Schema['StoryblokFilterQueryOperations'] | null;
    analytics_target: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokQuestionanswerItem': {
    __typename?: 'StoryblokQuestionanswerItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokQuestionanswerComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokQuestionanswerItems': {
    __typename?: 'StoryblokQuestionanswerItems';
    items?: Array<Schema['StoryblokQuestionanswerItem'] | null> | null;
    total: number | null;
  };
  'StoryblokQuizComponent': {
    __typename?: 'StoryblokQuizComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    questions: Schema['StoryblokBlockScalar'] | null;
  };
  'StoryblokQuizItem': {
    __typename?: 'StoryblokQuizItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokQuizComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokQuizItems': {
    __typename?: 'StoryblokQuizItems';
    items?: Array<Schema['StoryblokQuizItem'] | null> | null;
    total: number | null;
  };
  'StoryblokRateLimit': {
    __typename?: 'StoryblokRateLimit';
    maxCost: number;
  };
  'StoryblokSalesvendorComponent': {
    __typename?: 'StoryblokSalesvendorComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    id: string | null;
    name: string | null;
  };
  'StoryblokSalesvendorFilterQuery': {
    id: Schema['StoryblokFilterQueryOperations'] | null;
    name: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokSalesvendorItem': {
    __typename?: 'StoryblokSalesvendorItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokSalesvendorComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokSalesvendorItems': {
    __typename?: 'StoryblokSalesvendorItems';
    items?: Array<Schema['StoryblokSalesvendorItem'] | null> | null;
    total: number | null;
  };
  'StoryblokSeopageflagComponent': {
    __typename?: 'StoryblokSeopageflagComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    description: string | null;
    h1: string | null;
    page_title: string | null;
  };
  'StoryblokSeopageflagFilterQuery': {
    page_title: Schema['StoryblokFilterQueryOperations'] | null;
    description: Schema['StoryblokFilterQueryOperations'] | null;
    h1: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokSeopageflagItem': {
    __typename?: 'StoryblokSeopageflagItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokSeopageflagComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokSeopageflagItems': {
    __typename?: 'StoryblokSeopageflagItems';
    items?: Array<Schema['StoryblokSeopageflagItem'] | null> | null;
    total: number | null;
  };
  'StoryblokSpace': {
    __typename?: 'StoryblokSpace';
    domain: string;
    id: number;
    languageCodes: Array<string | null>;
    name: string;
    version: number;
  };
  'StoryblokSponconsponsorComponent': {
    __typename?: 'StoryblokSponconsponsorComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
    description: string | null;
    facebook?: Schema['StoryblokLink'] | null;
    instagram?: Schema['StoryblokLink'] | null;
    logo?: Schema['StoryblokAsset'] | null;
    name: string | null;
    pinterest?: Schema['StoryblokLink'] | null;
    twitter?: Schema['StoryblokLink'] | null;
    visit_url?: Schema['StoryblokLink'] | null;
    youtube?: Schema['StoryblokLink'] | null;
  };
  'StoryblokSponconsponsorFilterQuery': {
    name: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokSponconsponsorItem': {
    __typename?: 'StoryblokSponconsponsorItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokSponconsponsorComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokSponconsponsorItems': {
    __typename?: 'StoryblokSponconsponsorItems';
    items?: Array<Schema['StoryblokSponconsponsorItem'] | null> | null;
    total: number | null;
  };
  'StoryblokSponsoredcontentComponent': {
    __typename?: 'StoryblokSponsoredcontentComponent';
    _editable: string | null;
    _uid: string | null;
    adobe_page_name: string | null;
    articles: Schema['StoryblokBlockScalar'] | null;
    component: string | null;
    digioh_component_id: string | null;
    enable_digioh_signup: boolean | null;
    hero: Schema['StoryblokBlockScalar'] | null;
    introduction: Schema['StoryblokBlockScalar'] | null;
    is_no_index: boolean | null;
    metadata: Schema['StoryblokJsonScalar'] | null;
    sponsor?: Schema['StoryblokStory'] | null;
    tracking_click_tag: string | null;
    tracking_page_impression_tag: string | null;
  };
  'StoryblokSponsoredcontentFilterQuery': {
    is_no_index: Schema['StoryblokFilterQueryOperations'] | null;
    adobe_page_name: Schema['StoryblokFilterQueryOperations'] | null;
    sponsor: Schema['StoryblokFilterQueryOperations'] | null;
    enable_digioh_signup: Schema['StoryblokFilterQueryOperations'] | null;
    digioh_component_id: Schema['StoryblokFilterQueryOperations'] | null;
    tracking_click_tag: Schema['StoryblokFilterQueryOperations'] | null;
    tracking_page_impression_tag: Schema['StoryblokFilterQueryOperations'] | null;
  };
  'StoryblokSponsoredcontentItem': {
    __typename?: 'StoryblokSponsoredcontentItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokSponsoredcontentComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokSponsoredcontentItems': {
    __typename?: 'StoryblokSponsoredcontentItems';
    items?: Array<Schema['StoryblokSponsoredcontentItem'] | null> | null;
    total: number | null;
  };
  'StoryblokStory': {
    __typename?: 'StoryblokStory';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content: Schema['StoryblokJsonScalar'] | null;
    createdAt: string | null;
    firstPublishedAt: string | null;
    fullSlug: string | null;
    groupId: number | null;
    id: number | null;
    isStartpage: boolean | null;
    lang: string | null;
    metaData: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parentId: number | null;
    path: string | null;
    position: number | null;
    publishedAt: string | null;
    releaseId: number | null;
    slug: string | null;
    sortByDate: string | null;
    tagList: Array<string | null> | null;
    translatedSlugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokTableComponent': {
    __typename?: 'StoryblokTableComponent';
    _editable: string | null;
    _uid: string | null;
    component: string | null;
  };
  'StoryblokTableItem': {
    __typename?: 'StoryblokTableItem';
    alternates?: Array<Schema['StoryblokAlternate'] | null> | null;
    content?: Schema['StoryblokTableComponent'] | null;
    created_at: string | null;
    default_full_slug: string | null;
    first_published_at: string | null;
    full_slug: string | null;
    group_id: number | null;
    id: number | null;
    is_startpage: boolean | null;
    lang: string | null;
    meta_data: Schema['StoryblokJsonScalar'] | null;
    name: string | null;
    parent_id: number | null;
    path: string | null;
    position: number | null;
    published_at: string | null;
    release_id: number | null;
    slug: string | null;
    sort_by_date: string | null;
    tag_list: Array<string | null> | null;
    translated_slugs?: Array<Schema['StoryblokTranslatedSlug'] | null> | null;
    uuid: string | null;
  };
  'StoryblokTableItems': {
    __typename?: 'StoryblokTableItems';
    items?: Array<Schema['StoryblokTableItem'] | null> | null;
    total: number | null;
  };
  'StoryblokTag': {
    __typename?: 'StoryblokTag';
    name: string;
    taggingsCount: number;
  };
  'StoryblokTags': {
    __typename?: 'StoryblokTags';
    items?: Array<Schema['StoryblokTag']>;
  };
  'StoryblokTranslatedSlug': {
    __typename?: 'StoryblokTranslatedSlug';
    lang: string;
    name: string | null;
    path: string | null;
  };
};

import { ResolverFn } from '@grafbase/sdk'

export type Resolver = {
  'Query.hello': ResolverFn<Schema['Query'], { name: string | null,  }, string>
}

