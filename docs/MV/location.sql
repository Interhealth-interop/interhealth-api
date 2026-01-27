CREATE OR REPLACE VIEW "DBAMV"."LOCATION"
(
    "location_code",
    "location_place_code",
    "location_name",
    "location_type",
    "created_date",
    "updated_date",
    "location_reason",
    "location_responsible",
    "location_organization_code"
) AS
SELECT
    S.CD_SETOR                            AS "location_code",
    L.CD_LEITO                            AS "location_place_code",
    S.NM_SETOR || ' ' || L.DS_LEITO       AS "location_name",
    TA.DS_TIP_ACOM                        AS "location_type",
    TO_CHAR(S.DT_INCLUSAO, 'YYYY-MM-DD')  AS "created_date",
    NULL                                  AS "updated_date",
    NULL                                  AS "location_reason",
    NULL                                  AS "location_responsible",
    S.CD_MULTI_EMPRESA                    AS "location_organization_code"
FROM DBAMV.SETOR      S
  LEFT JOIN DBAMV.UNID_INT UI ON UI.CD_SETOR = S.CD_SETOR
  LEFT JOIN DBAMV.LEITO L ON L.CD_UNID_INT = UI.CD_UNID_INT
  LEFT JOIN DBAMV.TIP_ACOM TA ON TA.CD_TIP_ACOM = L.CD_TIP_ACOM;
